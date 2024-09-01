use std::{hash::Hash, ops::Deref, sync::Arc};

use gpui::{
    px, size, AppContext, Asset, Bounds, Element, Hitbox, ImageCacheError, InteractiveElement,
    Interactivity, IntoElement, IsZero, Pixels, RenderImage, SharedString, Size, StyleRefinement,
    Styled, WindowContext,
};
use image::Frame;
use smallvec::SmallVec;

use image::ImageBuffer;

#[derive(Debug, Clone, Hash)]
pub enum SvgSource {
    /// A svg bytes
    Data(Arc<[u8]>),
    /// An asset path
    Path(SharedString),
}

impl From<&[u8]> for SvgSource {
    fn from(data: &[u8]) -> Self {
        Self::Data(data.into())
    }
}

impl From<Arc<[u8]>> for SvgSource {
    fn from(data: Arc<[u8]>) -> Self {
        Self::Data(data)
    }
}

impl From<SharedString> for SvgSource {
    fn from(path: SharedString) -> Self {
        Self::Path(path)
    }
}

impl From<&'static str> for SvgSource {
    fn from(path: &'static str) -> Self {
        Self::Path(path.into())
    }
}

impl Clone for SvgImg {
    fn clone(&self) -> Self {
        Self {
            interactivity: Interactivity::default(),
            source: self.source.clone(),
            size: self.size,
        }
    }
}

enum Image {}

#[derive(Debug, Clone)]
struct ImageSource {
    source: SvgSource,
    size: Size<Pixels>,
}

impl Hash for ImageSource {
    /// Hash to to control the Asset cache
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.source.hash(state);
    }
}

impl Asset for Image {
    type Source = ImageSource;
    type Output = Result<Arc<RenderImage>, ImageCacheError>;

    fn load(
        source: Self::Source,
        cx: &mut AppContext,
    ) -> impl std::future::Future<Output = Self::Output> + Send + 'static {
        let scale = 2.;
        let asset_source = cx.asset_source().clone();

        async move {
            let size = source.size;
            if size.width.is_zero() || size.height.is_zero() {
                return Err(usvg::Error::InvalidSize.into());
            }
            let size = Size {
                width: (size.width * 2).ceil(),
                height: (size.height * scale).ceil(),
            };

            let bytes = match source.source {
                SvgSource::Data(data) => data,
                SvgSource::Path(path) => {
                    if let Ok(Some(data)) = asset_source.load(&path) {
                        data.deref().to_vec().into()
                    } else {
                        Err(std::io::Error::other(format!(
                            "failed to load svg image from path: {}",
                            path
                        )))
                        .map_err(|e| ImageCacheError::Io(Arc::new(e)))?
                    }
                }
            };

            let options = usvg::Options {
                ..Default::default()
            };
            let tree = usvg::Tree::from_data(&bytes, &options)?;

            let mut pixmap =
                resvg::tiny_skia::Pixmap::new(size.width.0 as u32, size.height.0 as u32)
                    .ok_or(usvg::Error::InvalidSize)?;

            let transform = tree.view_box().to_transform(
                resvg::tiny_skia::Size::from_wh(size.width.0, size.height.0)
                    .ok_or(usvg::Error::InvalidSize)?,
            );
            resvg::render(&tree, transform, &mut pixmap.as_mut());

            let mut buffer = ImageBuffer::from_raw(pixmap.width(), pixmap.height(), pixmap.take())
                .expect("invalid svg image buffer");

            // Convert from RGBA to BGRA.
            for pixel in buffer.chunks_exact_mut(4) {
                pixel.swap(0, 2);
            }

            Ok(Arc::new(RenderImage::new(SmallVec::from_elem(
                Frame::new(buffer),
                1,
            ))))
        }
    }
}

/// An SVG image element.
pub fn svg_img() -> SvgImg {
    SvgImg::new()
}

pub struct SvgImg {
    interactivity: Interactivity,
    source: Option<SvgSource>,
    size: Size<Pixels>,
}

impl SvgImg {
    /// Create a new svg image element.
    ///
    /// The `src_width` and `src_height` are the original width and height of the svg image.
    pub fn new() -> Self {
        Self {
            interactivity: Interactivity::default(),
            source: None,
            size: Size::default(),
        }
    }

    /// Set the path of the svg image from the asset.
    ///
    /// The `size` argument is the size of the original svg image.
    #[must_use]
    pub fn source(
        mut self,
        source: impl Into<SvgSource>,
        width: impl Into<Pixels>,
        height: impl Into<Pixels>,
    ) -> Self {
        self.source = Some(source.into());
        self.size = size(width.into(), height.into());
        self
    }
}

impl IntoElement for SvgImg {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for SvgImg {
    type RequestLayoutState = ();
    type PrepaintState = Option<Hitbox>;

    fn id(&self) -> Option<gpui::ElementId> {
        self.interactivity.element_id.clone()
    }

    fn request_layout(
        &mut self,
        global_id: Option<&gpui::GlobalElementId>,
        cx: &mut WindowContext,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        let layout_id = self
            .interactivity
            .request_layout(global_id, cx, |style, cx| cx.request_layout(style, None));

        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        global_id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        _: &mut Self::RequestLayoutState,
        cx: &mut WindowContext,
    ) -> Self::PrepaintState {
        self.interactivity
            .prepaint(global_id, bounds, bounds.size, cx, |_, _, hitbox, _| hitbox)
    }

    fn paint(
        &mut self,
        global_id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        _: &mut Self::RequestLayoutState,
        hitbox: &mut Self::PrepaintState,
        cx: &mut WindowContext,
    ) {
        let source = self.source.clone();

        self.interactivity
            .paint(global_id, bounds, hitbox.as_ref(), cx, |_style, cx| {
                let size = self.size;

                let data = if let Some(source) = source {
                    match cx.use_asset::<Image>(&ImageSource { source, size }) {
                        Some(Ok(data)) => Some(data),
                        _ => None,
                    }
                } else {
                    None
                };

                if let Some(data) = data {
                    // To calculate the ratio of the original image size to the container bounds size.
                    // Scale by shortest side (width or height) to get a fit image.
                    // And center the image in the container bounds.
                    let ratio = if bounds.size.width < bounds.size.height {
                        bounds.size.width / size.width
                    } else {
                        bounds.size.height / size.height
                    };

                    let ratio = ratio.min(1.0);

                    let new_size = gpui::Size {
                        width: size.width * ratio,
                        height: size.height * ratio,
                    };
                    let new_origin = gpui::Point {
                        x: bounds.origin.x + px(((bounds.size.width - new_size.width) / 2.).into()),
                        y: bounds.origin.y
                            + px(((bounds.size.height - new_size.height) / 2.).into()),
                    };

                    let img_bounds = Bounds {
                        origin: new_origin.map(|origin| origin.floor()),
                        size: new_size.map(|size| size.ceil()),
                    };

                    match cx.paint_image(img_bounds, px(0.).into(), data, 0, false) {
                        Ok(_) => {}
                        Err(err) => eprintln!("failed to paint svg image: {:?}", err),
                    }
                }
            })
    }
}

impl Styled for SvgImg {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.interactivity.base_style
    }
}

impl InteractiveElement for SvgImg {
    fn interactivity(&mut self) -> &mut Interactivity {
        &mut self.interactivity
    }
}

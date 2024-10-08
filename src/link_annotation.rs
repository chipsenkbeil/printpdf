use crate::{PdfPageIndex, Rect};
use lopdf::{self, Dictionary, Object};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LinkAnnotation {
    pub rect: Rect,
    pub border: BorderArray,
    pub c: ColorArray,
    pub a: Actions,
    pub h: HighlightingMode,
}

impl LinkAnnotation {
    /// Creates a new LinkAnnotation
    pub fn new(
        rect: Rect,
        border: Option<BorderArray>,
        c: Option<ColorArray>,
        a: Actions,
        h: Option<HighlightingMode>,
    ) -> Self {
        Self {
            rect,
            border: border.unwrap_or_default(),
            c: c.unwrap_or_default(),
            a,
            h: h.unwrap_or_default(),
        }
    }

    pub fn into_object(self, ctx: AnnotationContext) -> Object {
        let mut dict = Dictionary::new();
        dict.set("Type", Object::Name("Annot".as_bytes().to_vec()));
        dict.set("Subtype", Object::Name("Link".as_bytes().to_vec()));
        dict.set(
            "Rect",
            Object::Array(vec![
                self.rect.ll.x.into(),
                self.rect.ll.y.into(),
                self.rect.ur.x.into(),
                self.rect.ur.y.into(),
            ]),
        );

        dict.set::<&str, Object>("A", self.a.into_object(ctx));
        dict.set::<&str, Object>("Border", self.border.into());
        dict.set::<&str, Object>("C", self.c.into());
        dict.set::<&str, Object>("H", self.h.into());

        Object::Dictionary(dict)
    }
}

#[derive(Debug, Clone)]
pub struct AnnotationContext<'a> {
    pub page_id_to_obj: &'a HashMap<usize, (u32, u16)>,
}

#[derive(Debug, Clone)]
pub enum BorderArray {
    Solid([f32; 3]),
    Dashed([f32; 3], DashPhase),
}

impl Default for BorderArray {
    fn default() -> Self {
        BorderArray::Solid([0.0, 0.0, 1.0])
    }
}

impl Into<Object> for BorderArray {
    fn into(self) -> Object {
        match self {
            BorderArray::Solid(arr) => Object::Array(vec![
                Object::Real(arr[0].into()),
                Object::Real(arr[1].into()),
                Object::Real(arr[2].into()),
            ]),
            BorderArray::Dashed(arr, phase) => Object::Array(vec![
                Object::Real(arr[0].into()),
                Object::Real(arr[1].into()),
                Object::Real(arr[2].into()),
                Object::Real(phase.phase.into()),
            ]),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DashPhase {
    pub dash_array: Vec<f32>,
    pub phase: f32,
}

impl Into<Object> for DashPhase {
    fn into(self) -> Object {
        Object::Array(vec![
            Object::Array(
                self.dash_array
                    .into_iter()
                    .map(|x| Object::Real(x.into()))
                    .collect(),
            ),
            Object::Real(self.phase.into()),
        ])
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ColorArray {
    Transparent,
    Gray([f32; 1]),
    RGB([f32; 3]),
    CMYK([f32; 4]),
}

impl Default for ColorArray {
    fn default() -> Self {
        ColorArray::RGB([0.0, 1.0, 1.0])
    }
}

impl Into<Object> for ColorArray {
    fn into(self) -> Object {
        match self {
            ColorArray::Transparent => Object::Array(vec![]),
            ColorArray::Gray(arr) => Object::Array(vec![Object::Real(arr[0].into())]),
            ColorArray::RGB(arr) => Object::Array(vec![
                Object::Real(arr[0].into()),
                Object::Real(arr[1].into()),
                Object::Real(arr[2].into()),
            ]),
            ColorArray::CMYK(arr) => Object::Array(vec![
                Object::Real(arr[0].into()),
                Object::Real(arr[1].into()),
                Object::Real(arr[2].into()),
                Object::Real(arr[3].into()),
            ]),
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Destination {
    /// Display `page` with coordinates `top` and `left` positioned at the upper-left corner of the
    /// window and the contents of the page magnified by `zoom`.
    ///
    /// A value of `None` for any parameter indicates to leave the current value unchanged, and a
    /// `zoom` value of 0 has the same meaning as `None`.
    XYZ {
        page: PdfPageIndex,
        left: Option<f32>,
        top: Option<f32>,
        zoom: Option<f32>,
    },
}

#[derive(Debug, Clone)]
pub enum Actions {
    GoTo(Destination),
    URI(String),
}

impl Actions {
    pub fn go_to(destination: Destination) -> Self {
        Self::GoTo(destination)
    }

    pub fn uri(uri: String) -> Self {
        Self::URI(uri)
    }

    pub fn into_object(self, ctx: AnnotationContext) -> Object {
        let mut dict = Dictionary::new();
        match self {
            Self::GoTo(destination) => {
                dict.set("S", Object::Name("GoTo".as_bytes().to_vec()));
                dict.set(
                    "D",
                    match destination {
                        Destination::XYZ {
                            page,
                            left,
                            top,
                            zoom,
                        } => Object::Array(vec![
                            Object::Reference(
                                ctx.page_id_to_obj
                                    .get(&page.0)
                                    .expect("page index should be valid object")
                                    .to_owned(),
                            ),
                            "XYZ".into(),
                            left.map(Object::Real).unwrap_or(Object::Null),
                            top.map(Object::Real).unwrap_or(Object::Null),
                            zoom.map(Object::Real).unwrap_or(Object::Null),
                        ]),
                    },
                );
            }
            Self::URI(uri) => {
                dict.set("S", Object::Name("URI".as_bytes().to_vec()));
                dict.set(
                    "URI",
                    Object::String(uri.into_bytes().to_vec(), lopdf::StringFormat::Literal),
                );
            }
        }
        Object::Dictionary(dict)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HighlightingMode {
    None,
    Invert,
    Outline,
    Push,
}

impl Default for HighlightingMode {
    fn default() -> Self {
        HighlightingMode::Invert
    }
}

impl Into<Object> for HighlightingMode {
    fn into(self) -> Object {
        match self {
            HighlightingMode::None => Object::Name("N".as_bytes().to_vec()),
            HighlightingMode::Invert => Object::Name("I".as_bytes().to_vec()),
            HighlightingMode::Outline => Object::Name("O".as_bytes().to_vec()),
            HighlightingMode::Push => Object::Name("P".as_bytes().to_vec()),
        }
    }
}

/// Named reference to a LinkAnnotation
#[derive(Debug)]
pub struct LinkAnnotationRef {
    pub(crate) name: String,
}

impl LinkAnnotationRef {
    pub fn new(index: usize) -> Self {
        Self {
            name: format!("PT{index}"),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct LinkAnnotationList {
    link_annotations: HashMap<String, LinkAnnotation>,
}

impl IntoIterator for LinkAnnotationList {
    type Item = (String, LinkAnnotation);
    type IntoIter = std::collections::hash_map::IntoIter<String, LinkAnnotation>;

    fn into_iter(self) -> Self::IntoIter {
        self.link_annotations.into_iter()
    }
}

impl LinkAnnotationList {
    /// Creates a new LinkAnnotation list
    pub fn new() -> Self {
        Self {
            link_annotations: HashMap::new(),
        }
    }

    /// Adds a new LinkAnnotation to the LinkAnnotation list
    pub fn add_link_annotation(&mut self, link_annotation: LinkAnnotation) -> LinkAnnotationRef {
        let len = self.link_annotations.len();
        let link_annotation_ref = LinkAnnotationRef::new(len);
        self.link_annotations
            .insert(link_annotation_ref.name.clone(), link_annotation);
        link_annotation_ref
    }
}

impl From<LinkAnnotationList> for Dictionary {
    fn from(_val: LinkAnnotationList) -> Self {
        if _val.link_annotations.is_empty() {
            return Dictionary::new();
        }

        let mut dict = Dictionary::new();
        dict.set("Type", Object::Name("Annot".as_bytes().to_vec()));
        dict.set("Subtype", Object::Name("Link".as_bytes().to_vec()));
        dict.set(
            "Rect",
            Object::Array(vec![
                _val.link_annotations["PT0"].rect.ll.x.into(),
                _val.link_annotations["PT0"].rect.ll.y.into(),
                _val.link_annotations["PT0"].rect.ur.x.into(),
                _val.link_annotations["PT0"].rect.ur.y.into(),
            ]),
        );
        dict
    }
}

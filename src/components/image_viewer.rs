// Copied this code from iced::widget::image::viewer to add some features:
// - zoom and offset shouldn't reset when this widget is rerendered
// => moved scale and offset variable out of the element's inner scope

//! Zoom and pan on an image.
use iced::advanced::renderer::Quad;
#[warn(unused)]
use iced::border;
use iced::advanced::image::{self, FilterMethod, Image};
use iced::advanced::{Clipboard, Layout, layout, Shell, Widget, renderer};
use iced::{Color, mouse};
use iced::advanced::widget::tree::{self, Tree};
use iced::{ ContentFit, Element, Event, Length, Point, Radians, Rectangle, Size, Vector };



/// A frame that displays an image with the ability to zoom in/out and pan.
pub struct Viewer<'a, Handle, Message> {
    width: Length,
    height: Length,
    offset: Vector,
    opacity: f32,
    scale: f32,
    min_scale: f32,
    max_scale: f32,
    scale_step: f32,
    handle: Handle,
    filter_method: FilterMethod,
    content_fit: ContentFit,
    on_scale: Box<dyn Fn(f32) -> Message + 'a>,
    on_offset: Box<dyn Fn(Vector) -> Message + 'a>,
}

impl<'a, Handle, Message> Viewer<'a, Handle, Message> {
    /// Creates a new [`Viewer`] with the given [`State`].
    pub fn new<T, F1, F2>(
        handle: T,
        width: impl Into<Length>,
        height: impl Into<Length>,
        scale: f32,
        offset: Vector,
        opacity: f32,
        on_scale: F1,
        on_offset: F2
    ) -> Self
    where
        T: Into<Handle>,
        F1: Fn(f32) -> Message + 'a,
        F2: Fn(Vector) -> Message + 'a,
    {
        Viewer {
            handle: handle.into(),
            width: width.into(),
            height: height.into(),
            offset,
            opacity,
            scale,
            min_scale: 0.25,
            max_scale: 10.0,
            scale_step: 0.10,
            filter_method: FilterMethod::default(),
            content_fit: ContentFit::default(),
            on_scale: Box::new(on_scale),
            on_offset: Box::new(on_offset),
        }
    }
}

impl<'a, Message, Theme, Renderer, Handle> Widget<Message, Theme, Renderer> for Viewer<'a, Handle, Message>
where
    Renderer: image::Renderer<Handle = Handle>,
    Handle: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        // The raw w/h of the underlying image
        let image_size =
            renderer.measure_image(&self.handle).unwrap_or_default();

        let image_size =
            Size::new(image_size.width as f32, image_size.height as f32);

        // The size to be available to the widget prior to `Shrink`ing
        let raw_size = limits.resolve(self.width, self.height, image_size);

        // The uncropped size of the image when fit to the bounds above
        let full_size = self.content_fit.fit(image_size, raw_size);

        // Shrink the widget to fit the resized image, if requested
        let final_size = Size {
            width: match self.width {
                Length::Shrink => f32::min(raw_size.width, full_size.width),
                _ => raw_size.width,
            },
            height: match self.height {
                Length::Shrink => f32::min(raw_size.height, full_size.height),
                _ => raw_size.height,
            },
        };

        layout::Node::new(final_size)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let Some(cursor_position) = cursor.position_over(bounds)
                else { return };

                match *delta {
                    mouse::ScrollDelta::Lines { y, .. } |
                    mouse::ScrollDelta::Pixels { y, .. } => {
                        if y < 0.0 && self.scale > self.min_scale
                        || y > 0.0 && self.scale < self.max_scale
                        {
                            let new_scale = (if y > 0.0 {
                                self.scale * (1.0 + self.scale_step)
                            } else {
                                self.scale / (1.0 + self.scale_step)
                            })
                            .clamp(self.min_scale, self.max_scale);

                            let scaled_size = scaled_image_size(
                                renderer,
                                &self.handle,
                                new_scale,
                                bounds.size(),
                                self.content_fit,
                            );

                            let factor = new_scale / self.scale - 1.0;

                            let cursor_to_center = cursor_position - bounds.center();

                            let adjustment = cursor_to_center * factor
                                           + self.offset * factor;

                            let new_offset = Vector::new(
                                if scaled_size.width > bounds.width {
                                    self.offset.x + adjustment.x
                                }
                                else { 0.0 },
                                if scaled_size.height > bounds.height {
                                    self.offset.y + adjustment.y
                                }
                                else { 0.0 },
                            );

                            let update_scale = &self.on_scale;
                            shell.publish( update_scale(new_scale) );
                            let update_offset = &self.on_offset;
                            shell.publish( update_offset(new_offset) );
                        }
                    }
                }

                shell.request_redraw();
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(cursor_position) = cursor.position_over(bounds)
                else { return; };

                let state = tree.state.downcast_mut::<State>();

                state.cursor_grabbed_at = Some(cursor_position);
                state.starting_offset = self.offset;

                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let state = tree.state.downcast_mut::<State>();

                state.cursor_grabbed_at = None;
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                let state = tree.state.downcast_mut::<State>();

                if let Some(origin) = state.cursor_grabbed_at {
                    let scaled_size = scaled_image_size(
                        renderer,
                        &self.handle,
                        self.scale,
                        bounds.size(),
                        self.content_fit,
                    );

                    let hidden_width = (scaled_size.width - bounds.width / 2.0)
                        .max(0.0)
                        .round();
                    let hidden_height = (scaled_size.height - bounds.height / 2.0)
                        .max(0.0)
                        .round();

                    let delta = *position - origin;

                    let x = if bounds.width < scaled_size.width {
                        (state.starting_offset.x - delta.x)
                            .clamp(-hidden_width, hidden_width)
                    }
                    else { 0.0 };

                    let y = if bounds.height < scaled_size.height {
                        (state.starting_offset.y - delta.y)
                            .clamp(-hidden_height, hidden_height)
                    }
                    else { 0.0 };

                    let update_offset = &self.on_offset;
                    shell.publish( update_offset(Vector::new(x, y)) );
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let is_mouse_over = cursor.is_over(bounds);

        if state.is_cursor_grabbed() {
            mouse::Interaction::Grabbing
        } else if is_mouse_over {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::None
        }
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        //let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        let final_size = scaled_image_size(
            renderer,
            &self.handle,
            self.scale,
            bounds.size(),
            self.content_fit,
        );

        let translation = {
            let diff_w = bounds.width - final_size.width;
            let diff_h = bounds.height - final_size.height;

            let image_top_left = match self.content_fit {
                ContentFit::None => {
                    Vector::new(diff_w.max(0.0) / 2.0, diff_h.max(0.0) / 2.0)
                }
                _ => Vector::new(diff_w / 2.0, diff_h / 2.0),
            };

            let offset = {
                let hidden_width = (final_size.width - bounds.width / 2.0)
                    .max(0.0).round();
                let hidden_height = (final_size.height - bounds.height / 2.0)
                    .max(0.0).round();

                Vector::new(
                    self.offset.x.clamp(-hidden_width, hidden_width),
                    self.offset.y.clamp(-hidden_height, hidden_height),
                )
            };

            image_top_left - offset
        };

        let drawing_bounds = Rectangle::new(bounds.position(), final_size);

        let render = |renderer: &mut Renderer| {
            renderer.with_translation(translation, |renderer| {
                // Draw a black background as an underlay for the image
                renderer.fill_quad(
                    Quad {
                        bounds: drawing_bounds,
                        ..Default::default()
                    },
                    Color::from_rgb(0.0, 0.0, 0.0)
                );
                // Draw the image
                renderer.draw_image(
                    Image {
                        handle: self.handle.clone(),
                        border_radius: border::Radius::default(),
                        filter_method: self.filter_method,
                        rotation: Radians(0.0),
                        opacity: self.opacity,
                        snap: true,
                    },
                    drawing_bounds,
                    *viewport - translation,
                );
            });
        };

        renderer.with_layer(bounds, render);
    }
}

/// The local state of a [`Viewer`].
#[derive(Debug, Clone, Copy)]
pub struct State {
    starting_offset: Vector,
    cursor_grabbed_at: Option<Point>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            starting_offset: Vector::default(),
            cursor_grabbed_at: None,
        }
    }
}

impl State {
    pub fn new() -> Self {
        State::default()
    }

    pub fn is_cursor_grabbed(&self) -> bool {
        self.cursor_grabbed_at.is_some()
    }
}

impl<'a, Message, Theme, Renderer, Handle> From<Viewer<'a, Handle, Message>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: 'a + image::Renderer<Handle = Handle>,
    Message: 'a,
    Handle: Clone + 'a,
{
    fn from(viewer: Viewer<'a, Handle, Message>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(viewer)
    }
}

/// Returns the bounds of the underlying image, given the bounds of
/// the [`Viewer`]. Scaling will be applied and original aspect ratio
/// will be respected.
pub fn scaled_image_size<Renderer>(
    renderer: &Renderer,
    handle: &<Renderer as image::Renderer>::Handle,
    scale: f32,
    bounds: Size,
    content_fit: ContentFit,
) -> Size
where
    Renderer: image::Renderer,
{
    let Size { width, height } =
        renderer.measure_image(handle).unwrap_or_default();

    let image_size = Size::new(width as f32, height as f32);

    let adjusted_fit = content_fit.fit(image_size, bounds);

    Size::new(
        adjusted_fit.width * scale,
        adjusted_fit.height * scale,
    )
}

use gpui::*;

pub struct TextInput {
    focus_handle: FocusHandle,
    value: SharedString,
    placeholder: SharedString,
    on_enter: Option<Box<dyn Fn(&str) + 'static>>,
}

impl TextInput {
    pub fn new(cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            value: "".into(),
            placeholder: "".into(),
            on_enter: None,
        }
    }

    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn value(&self) -> String {
        self.value.to_string()
    }

    pub fn set_value(&mut self, value: impl Into<SharedString>) {
        self.value = value.into();
    }

    pub fn clear(&mut self) {
        self.value = "".into();
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    pub fn on_enter<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + 'static,
    {
        self.on_enter = Some(Box::new(callback));
        self
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

pub struct TextInputView {
    input: TextInput,
}

impl TextInputView {
    pub fn new(cx: &mut App) -> Self {
        Self {
            input: TextInput::new(cx),
        }
    }

    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.input.placeholder = text.into();
        self
    }

    pub fn on_enter<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + 'static,
    {
        self.input = self.input.on_enter(callback);
        self
    }

    pub fn value(&self) -> String {
        self.input.value()
    }

    pub fn set_value(&mut self, value: impl Into<SharedString>) {
        self.input.set_value(value);
    }

    pub fn clear(&mut self) {
        self.input.clear();
    }
}

impl Focusable for TextInputView {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input.focus_handle(cx)
    }
}

impl Render for TextInputView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.input.focus_handle.is_focused(window);

        div()
            .id("text-input")
            .track_focus(&self.input.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                // Gestion de Ctrl+V pour paste
                if event.keystroke.modifiers.control && event.keystroke.key == "v" {
                    // Essayer de lire le presse-papier
                    cx.read_from_clipboard().map(|clipboard_item| {
                        if let Some(text) = clipboard_item.text() {
                            let mut new_value = this.input.value.to_string();
                            new_value.push_str(&text);
                            this.input.value = new_value.into();
                            cx.notify();
                        }
                    });
                    return;
                }

                // Gestion de Enter pour valider
                if event.keystroke.key == "enter" {
                    let value = this.input.value.to_string();
                    if let Some(ref callback) = this.input.on_enter {
                        callback(&value);
                    }
                    return;
                }

                // Gestion des caractères normaux
                if event.keystroke.key.as_str().len() == 1 && !event.keystroke.modifiers.control {
                    let mut new_value = this.input.value.to_string();
                    new_value.push_str(&event.keystroke.key);
                    this.input.value = new_value.into();
                    cx.notify();
                } else if event.keystroke.key == "backspace" {
                    let mut new_value = this.input.value.to_string();
                    new_value.pop();
                    this.input.value = new_value.into();
                    cx.notify();
                } else if event.keystroke.key == "space" {
                    let mut new_value = this.input.value.to_string();
                    new_value.push(' ');
                    this.input.value = new_value.into();
                    cx.notify();
                }
            }))
            .flex()
            .items_center()
            .w_full()
            .h_full()
            .px_3()
            .child(
                if self.input.value.is_empty() {
                    div()
                        .text_color(rgb(0x888888))
                        .child(self.input.placeholder.clone())
                } else {
                    div()
                        .text_color(if focused { rgb(0xffffff) } else { rgb(0xcccccc) })
                        .child(self.input.value.clone())
                }
            )
    }
}


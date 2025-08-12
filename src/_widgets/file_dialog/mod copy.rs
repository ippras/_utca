#[cfg(not(target_arch = "wasm32"))]
pub use self::native::FileDialog;
#[cfg(target_arch = "wasm32")]
pub use self::web::FileDialog;

type Content = Vec<u8>;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::{some, Content};
    use std::{
        fs::{read, write},
        io::Result,
    };

    #[derive(Debug, Default)]
    pub struct FileDialog {
        file: Option<Content>,
    }

    impl FileDialog {
        pub fn load(&mut self) -> Result<()> {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                self.file = Some(read(path)?);
            }
            Ok(())
        }

        pub fn save(&self, file_name: &str, content: Content) -> Result<()> {
            if let Some(path) = rfd::FileDialog::new().set_file_name(file_name).save_file() {
                write(path, content)?;
            }
            Ok(())
        }

        pub fn take(&mut self) -> Option<Content> {
            self.file.take_if(some)
        }
    }
}

/// https://stackoverflow.com/questions/3665115/how-to-create-a-file-in-memory-for-user-to-download-but-not-through-server
// #[cfg(target_arch = "wasm32")]
mod web {
    use super::{some, Content};
    use js_sys::{Array, ArrayBuffer, Uint8Array};
    use std::sync::mpsc::{channel, Receiver, Sender};
    use wasm_bindgen::{prelude::*, JsCast, JsError, JsValue};
    use web_sys::{window, File, FilePropertyBag, FileReader, HtmlInputElement, Url};

    pub struct FileDialog {
        tx: Sender<Content>,
        rx: Receiver<Content>,
        input: HtmlInputElement,
        closure: Option<Closure<dyn FnMut()>>,
    }

    impl FileDialog {
        pub fn new() -> Result<Self, JsValue> {
            let (tx, rx) = channel();
            let window = window().ok_or(JsError::new("window not found"))?;
            let document = window
                .document()
                .ok_or(JsError::new("document not found"))?;
            let body = document.body().ok_or(JsError::new("body not found"))?;
            let input = document
                .create_element("input")?
                .dyn_into::<HtmlInputElement>()?;
            // input.set_attribute("type", "file")?;
            input.set_type("file");
            input.set_multiple(true);
            input.style().set_property("display", "none")?;
            body.append_child(&input)?;
            Ok(Self {
                rx,
                tx,
                input,
                closure: None,
            })
        }
    }

    impl Default for FileDialog {
        fn default() -> Self {
            Self::new().expect("create default file dialog")
        }
    }

    impl Drop for FileDialog {
        fn drop(&mut self) {
            self.input.remove();
            if let Some(closure) = self.closure.take_if(some) {
                closure.forget();
            }
        }
    }

    impl FileDialog {
        pub fn load(&mut self) {
            if let Some(closure) = self.closure.take_if(some) {
                self.input
                    .remove_event_listener_with_callback("change", closure.as_ref().unchecked_ref())
                    .unwrap();
                closure.forget();
            }
            let tx = self.tx.clone();
            let input = self.input.clone();
            let closure = Closure::once(move || {
                if let Some(file) = input.files().and_then(|files| files.get(0)) {
                    let reader = FileReader::new().unwrap();
                    let reader_clone = reader.clone();
                    let onload_closure = Closure::once(Box::new(move || {
                        let array_buffer = reader_clone
                            .result()
                            .unwrap()
                            .dyn_into::<ArrayBuffer>()
                            .unwrap();
                        let buffer = Uint8Array::new(&array_buffer).to_vec();
                        tx.send(buffer).ok();
                    }));
                    reader.set_onload(Some(onload_closure.as_ref().unchecked_ref()));
                    reader.read_as_array_buffer(&file).unwrap();
                    onload_closure.forget();
                }
            });
            self.input
                .add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())
                .unwrap();
            self.closure = Some(closure);
            self.input.click();
        }

        pub fn save(&self, file_name: &str, content: Content) -> Result<(), JsValue> {
            if let Some(window) = window() {
                // let array = Uint8Array::from(&*content);
                // let file_bits = Array::new();
                // file_bits.push(&array.buffer());
                let file_bits = Array::from(&Uint8Array::from(&*content));
                let file = File::new_with_blob_sequence_and_options(
                    &file_bits.into(),
                    file_name,
                    FilePropertyBag::new().type_("application/octet-stream"),
                )?;
                let url = Url::create_object_url_with_blob(&file)?;
                window.location().set_href(&url)?;
            }
            Ok(())
        }

        pub fn take(&self) -> Option<Content> {
            self.rx.try_recv().ok()
        }
    }
}

fn some<T>(_: T) -> bool {
    true
}

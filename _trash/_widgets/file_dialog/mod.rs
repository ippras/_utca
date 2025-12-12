#[cfg(not(target_arch = "wasm32"))]
pub use self::native::FileDialog;
#[cfg(target_arch = "wasm32")]
pub use self::web::FileDialog;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::some;
    use std::{
        fs::{read, write},
        io::Result,
    };

    #[derive(Debug, Default)]
    pub struct FileDialog {
        file: Option<Vec<u8>>,
    }

    impl FileDialog {
        pub fn load(&mut self) -> Result<()> {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                self.file = Some(read(path)?);
            }
            Ok(())
        }

        pub fn save<T: AsRef<[u8]>>(&self, file_name: &str, bytes: T) -> Result<()> {
            if let Some(path) = rfd::FileDialog::new().set_file_name(file_name).save_file() {
                write(path, bytes)?;
            }
            Ok(())
        }

        pub fn take(&mut self) -> Option<Vec<u8>> {
            self.file.take_if(some)
        }
    }
}

/// https://stackoverflow.com/questions/3665115/how-to-create-a-file-in-memory-for-user-to-download-but-not-through-server
#[cfg(target_arch = "wasm32")]
mod web {
    use super::some;
    use js_sys::{Array, ArrayBuffer, Uint8Array};
    use std::{
        rc::Rc,
        sync::mpsc::{channel, Receiver, Sender},
    };
    use wasm_bindgen::{prelude::*, JsCast, JsError, JsValue};
    use web_sys::{
        window, Blob, BlobPropertyBag, File, FilePropertyBag, FileReader, HtmlAnchorElement,
        HtmlInputElement, Url,
    };

    enum Io {
        Input(HtmlInputElement),
        Output(HtmlAnchorElement),
    }

    // pub enum FileDialog {
    //     Input {
    //         tx: Sender<Vec<u8>>,
    //         rx: Receiver<Vec<u8>>,
    //         input: HtmlInputElement,
    //         callback: Option<Closure<dyn FnMut() -> Result<(), JsValue>>>,
    //     }
    //     Output {
    //         output: HtmlAnchorElement,
    //     }
    // }

    pub struct FileDialog {
        tx: Sender<Vec<u8>>,
        rx: Receiver<Vec<u8>>,
        input: HtmlInputElement,
        output: HtmlAnchorElement,
        callback: Option<Closure<dyn FnMut() -> Result<(), JsValue>>>,
    }

    impl FileDialog {
        pub fn new() -> Result<Self, JsValue> {
            let (tx, rx) = channel();
            let window = window().expect("window not found");
            let document = window.document().expect("document not found");
            let body = document.body().expect("body not found");
            let input: HtmlInputElement = document.create_element("input")?.dyn_into()?;
            // input.set_attribute("type", "file")?;
            input.set_type("file");
            input.style().set_property("display", "none")?;
            body.append_child(&input)?;
            let output: HtmlAnchorElement = document.create_element("a")?.dyn_into()?;
            output.style().set_property("display", "none")?;
            Ok(Self {
                rx,
                tx,
                input,
                output,
                callback: None,
            })
        }

        pub fn single(&mut self) {
            self.input.set_multiple(false);
        }

        pub fn multiple(&mut self) {
            self.input.set_multiple(true);
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
            self.output.remove();
            if let Some(closure) = self.callback.take_if(some) {
                closure.forget();
            }
        }
    }

    impl FileDialog {
        pub fn load(&mut self) -> Result<(), JsValue> {
            if let Some(closure) = self.callback.take_if(some) {
                self.input
                    .remove_event_listener_with_callback("change", closure.as_ref().unchecked_ref())
                    .unwrap();
                closure.forget();
            }
            let tx = self.tx.clone();
            let input = self.input.clone();
            let multiple = self.input.multiple();
            let callback = Closure::once(move || {
                if let Some(file) = input.files().and_then(|files| files.get(0)) {
                    let reader = Rc::new(FileReader::new()?);
                    let onload: Closure<dyn FnMut() -> Result<(), JsValue>> = Closure::once({
                        let reader = reader.clone();
                        move || {
                            let array_buffer = reader.result()?.dyn_into::<ArrayBuffer>()?;
                            let bytes = Uint8Array::new(&array_buffer).to_vec();
                            tx.send(bytes).map_err(JsError::from)?;
                            Ok(())
                        }
                    });
                    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                    reader.read_as_array_buffer(&file)?;
                    onload.forget();
                }
                Ok(())
            });
            self.input
                .add_event_listener_with_callback("change", callback.as_ref().unchecked_ref())
                .unwrap();
            self.callback = Some(callback);
            self.input.click();
            Ok(())
        }

        // save<T: AsRef<[u8]>>(&self, file_name: &str, bytes: T)
        pub fn save<T: AsRef<[u8]>>(&self, file_name: &str, content: T) -> Result<(), JsValue> {
            if let Some(window) = window() {
                // let array = Uint8Array::from(&*content);
                // let file_bits = Array::new();
                // file_bits.push(&array.buffer());
                // let array = Array::from(&Uint8Array::from(bytes.as_ref()));
                // let array = Array::from(&Uint8Array::new(
                //     &unsafe { Uint8Array::view(content.as_ref()) }.into(),
                // ));

                // [see](https://stackoverflow.com/questions/69556755/web-sysurlcreate-object-url-with-blobblob-not-formatting-binary-data-co)
                let bytes = Uint8Array::from(content.as_ref());
                let array = Array::new();
                array.push(&bytes.buffer());

                let file = File::new_with_blob_sequence_and_options(
                    &array,
                    file_name,
                    FilePropertyBag::new().type_("application/octet-stream"),
                )?;
                // let blob = Blob::new_with_u8_array_sequence_and_options(
                //     &array,
                //     BlobPropertyBag::new().type_("application/octet-stream"),
                // )?;
                let url = Url::create_object_url_with_blob(&file)?;
                self.output.set_href(&url);
                self.output.set_download(&file_name);
                self.output.click();
            }
            Ok(())
        }

        pub fn take(&self) -> Option<Vec<u8>> {
            self.rx.try_recv().ok()
        }
    }
}

fn some<T>(_: &mut T) -> bool {
    true
}

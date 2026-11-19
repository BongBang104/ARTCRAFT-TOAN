//! Android clipboard implementation using `ndk-context` and JNI.
//!
//! Android exposes no native clipboard API to Rust, so this reaches the system clipboard
//! (`android.content.ClipboardManager`) directly through JNI. The JVM and the application
//! `Context` are published globally by `android-activity` through `ndk-context` before the
//! Rust `android_main` entry point runs, so they are always available here.

use crate::{common::Error, ImageData};
use std::borrow::Cow;
use std::ffi::OsString;
use std::path::PathBuf;

use jni::{
    jni_sig, jni_str,
    objects::{JObject, JString, JValue},
    JavaVM,
};

pub(crate) struct Clipboard;

impl Clipboard {
    pub(crate) fn new() -> Result<Self, Error> {
        // The JVM/Context are initialized by android-activity before the clipboard is used.
        let _ = ndk_context::android_context();
        Ok(Clipboard)
    }

    pub(crate) fn get_text(&mut self, _selection: ()) -> Result<String, Error> {
        let ctx = ndk_context::android_context();
        let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) };
        let res = vm.attach_current_thread(|env| -> jni::errors::Result<Option<String>> {
            let ctx_raw: jni::sys::jobject = ctx.context().cast();
            // The context is an Android global reference owned by ndk-context: borrow it, never delete.
            let context = unsafe { env.as_cast_raw::<JObject>(&ctx_raw)? };
            let service_obj = JObject::from(env.new_string("clipboard")?);
            let clipboard = env
                .call_method(
                    &*context,
                    jni_str!("getSystemService"),
                    jni_sig!("(Ljava/lang/String;)Ljava/lang/Object;"),
                    &[JValue::Object(&service_obj)],
                )?
                .l()?;
            if clipboard.is_null() {
                return Ok(None);
            }
            let clip = env
                .call_method(
                    &clipboard,
                    jni_str!("getPrimaryClip"),
                    jni_sig!("()Landroid/content/ClipData;"),
                    &[],
                )?
                .l()?;
            if clip.is_null() {
                return Ok(None);
            }
            let count = env
                .call_method(&clip, jni_str!("getItemCount"), jni_sig!("()I"), &[])?
                .i()?;
            if count <= 0 {
                return Ok(None);
            }
            let item = env
                .call_method(
                    &clip,
                    jni_str!("getItemAt"),
                    jni_sig!("(I)Landroid/content/ClipData$Item;"),
                    &[JValue::Int(0)],
                )?
                .l()?;
            if item.is_null() {
                return Ok(None);
            }
            let text = env
                .call_method(
                    &item,
                    jni_str!("getText"),
                    jni_sig!("()Ljava/lang/CharSequence;"),
                    &[],
                )?
                .l()?;
            if text.is_null() {
                return Ok(None);
            }
            let text = env
                .call_method(
                    &text,
                    jni_str!("toString"),
                    jni_sig!("()Ljava/lang/String;"),
                    &[],
                )?
                .l()?;
            if text.is_null() {
                return Ok(None);
            }
            let text = JString::cast_local(env, text)?;
            Ok(Some(text.try_to_string(env)?))
        });
        res.map_err(|_| Error::ContentNotAvailable)
            .and_then(|value| value.ok_or(Error::ContentNotAvailable))
    }

    pub(crate) fn set_text(&mut self, text: Cow<'_, str>) -> Result<(), Error> {
        let ctx = ndk_context::android_context();
        let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) };
        let res = vm.attach_current_thread(|env| -> jni::errors::Result<()> {
            let ctx_raw: jni::sys::jobject = ctx.context().cast();
            let context = unsafe { env.as_cast_raw::<JObject>(&ctx_raw)? };
            let service_obj = JObject::from(env.new_string("clipboard")?);
            let clipboard = env
                .call_method(
                    &*context,
                    jni_str!("getSystemService"),
                    jni_sig!("(Ljava/lang/String;)Ljava/lang/Object;"),
                    &[JValue::Object(&service_obj)],
                )?
                .l()?;
            if clipboard.is_null() {
                return Ok(());
            }
            let text_obj = JObject::from(env.new_string(&*text)?);
            let label_obj = JObject::from(env.new_string("")?);
            let clip = env
                .call_static_method(
                    jni_str!("android/content/ClipData"),
                    jni_str!("newPlainText"),
                    jni_sig!("(Ljava/lang/CharSequence;Ljava/lang/CharSequence;)Landroid/content/ClipData;"),
                    &[JValue::Object(&label_obj), JValue::Object(&text_obj)],
                )?
                .l()?;
            if clip.is_null() {
                return Ok(());
            }
            env.call_method(
                &clipboard,
                jni_str!("setPrimaryClip"),
                jni_sig!("(Landroid/content/ClipData;)V"),
                &[JValue::Object(&clip)],
            )?;
            Ok(())
        });
        res.map_err(|_| Error::ContentNotAvailable)
    }

    #[cfg(feature = "image-data")]
    pub(crate) fn get_image(&mut self, _selection: ()) -> Result<ImageData<'static>, Error> {
        Err(Error::ContentNotAvailable)
    }

    #[cfg(feature = "image-data")]
    pub(crate) fn set_image(&mut self, _image: ImageData<'_>) -> Result<(), Error> {
        Ok(())
    }

    /// The primary clip's HTML (`ClipData.Item.getHtmlText`), as other apps (Excel, Word,
    /// browsers, GridCraft) put a formatted copy there next to its plain text.
    pub(crate) fn get_html(&mut self, _selection: ()) -> Result<String, Error> {
        let ctx = ndk_context::android_context();
        let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) };
        let res = vm.attach_current_thread(|env| -> jni::errors::Result<Option<String>> {
            let ctx_raw: jni::sys::jobject = ctx.context().cast();
            // The context is an Android global reference owned by ndk-context: borrow it, never delete.
            let context = unsafe { env.as_cast_raw::<JObject>(&ctx_raw)? };
            let service_obj = JObject::from(env.new_string("clipboard")?);
            let clipboard = env
                .call_method(
                    &*context,
                    jni_str!("getSystemService"),
                    jni_sig!("(Ljava/lang/String;)Ljava/lang/Object;"),
                    &[JValue::Object(&service_obj)],
                )?
                .l()?;
            if clipboard.is_null() {
                return Ok(None);
            }
            let clip = env
                .call_method(
                    &clipboard,
                    jni_str!("getPrimaryClip"),
                    jni_sig!("()Landroid/content/ClipData;"),
                    &[],
                )?
                .l()?;
            if clip.is_null() {
                return Ok(None);
            }
            let count = env
                .call_method(&clip, jni_str!("getItemCount"), jni_sig!("()I"), &[])?
                .i()?;
            if count <= 0 {
                return Ok(None);
            }
            let item = env
                .call_method(
                    &clip,
                    jni_str!("getItemAt"),
                    jni_sig!("(I)Landroid/content/ClipData$Item;"),
                    &[JValue::Int(0)],
                )?
                .l()?;
            if item.is_null() {
                return Ok(None);
            }
            let html = env
                .call_method(
                    &item,
                    jni_str!("getHtmlText"),
                    jni_sig!("()Ljava/lang/String;"),
                    &[],
                )?
                .l()?;
            if html.is_null() {
                return Ok(None);
            }
            let html = JString::cast_local(env, html)?;
            Ok(Some(html.try_to_string(env)?))
        });
        res.map_err(|_| Error::ContentNotAvailable)
            .and_then(|value| value.filter(|h| !h.trim().is_empty()).ok_or(Error::ContentNotAvailable))
    }

    /// Put `html` on the clipboard with `alt` (or the HTML's text) as its plain-text form, through
    /// `ClipData.newHtmlText`, so apps that only read text still get the copy.
    pub(crate) fn set_html(&mut self, html: Cow<'_, str>, alt: Option<Cow<'_, str>>) -> Result<(), Error> {
        let text: String = match alt {
            Some(alt) => alt.into_owned(),
            None => strip_tags(&html),
        };
        let ctx = ndk_context::android_context();
        let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) };
        let res = vm.attach_current_thread(|env| -> jni::errors::Result<()> {
            let ctx_raw: jni::sys::jobject = ctx.context().cast();
            let context = unsafe { env.as_cast_raw::<JObject>(&ctx_raw)? };
            let service_obj = JObject::from(env.new_string("clipboard")?);
            let clipboard = env
                .call_method(
                    &*context,
                    jni_str!("getSystemService"),
                    jni_sig!("(Ljava/lang/String;)Ljava/lang/Object;"),
                    &[JValue::Object(&service_obj)],
                )?
                .l()?;
            if clipboard.is_null() {
                return Ok(());
            }
            let label_obj = JObject::from(env.new_string("ArtCraft")?);
            let text_obj = JObject::from(env.new_string(&*text)?);
            let html_obj = JObject::from(env.new_string(&*html)?);
            let clip = env
                .call_static_method(
                    jni_str!("android/content/ClipData"),
                    jni_str!("newHtmlText"),
                    jni_sig!("(Ljava/lang/CharSequence;Ljava/lang/CharSequence;Ljava/lang/String;)Landroid/content/ClipData;"),
                    &[JValue::Object(&label_obj), JValue::Object(&text_obj), JValue::Object(&html_obj)],
                )?
                .l()?;
            if clip.is_null() {
                return Ok(());
            }
            env.call_method(
                &clipboard,
                jni_str!("setPrimaryClip"),
                jni_sig!("(Landroid/content/ClipData;)V"),
                &[JValue::Object(&clip)],
            )?;
            Ok(())
        });
        res.map_err(|_| Error::ContentNotAvailable)
    }

    pub(crate) fn get_file_list(&mut self, _selection: ()) -> Result<Vec<PathBuf>, Error> {
        Err(Error::ContentNotAvailable)
    }

    pub(crate) fn set_file_list(&mut self, _files: Vec<OsString>) -> Result<(), Error> {
        Ok(())
    }
}

pub(crate) struct Get<'clipboard> {
    clipboard: &'clipboard mut Clipboard,
}

impl<'clipboard> Get<'clipboard> {
    pub(crate) fn new(clipboard: &'clipboard mut Clipboard) -> Self {
        Self { clipboard }
    }

    pub(crate) fn text(self) -> Result<String, Error> {
        self.clipboard.get_text(())
    }

    #[cfg(feature = "image-data")]
    pub(crate) fn image(self) -> Result<ImageData<'static>, Error> {
        Err(Error::ContentNotAvailable)
    }

    pub(crate) fn html(self) -> Result<String, Error> {
        self.clipboard.get_html(())
    }

    pub(crate) fn file_list(self) -> Result<Vec<PathBuf>, Error> {
        Err(Error::ContentNotAvailable)
    }
}

pub(crate) struct Set<'clipboard> {
    clipboard: &'clipboard mut Clipboard,
}

impl<'clipboard> Set<'clipboard> {
    pub(crate) fn new(clipboard: &'clipboard mut Clipboard) -> Self {
        Self { clipboard }
    }

    pub(crate) fn text(self, text: Cow<'_, str>) -> Result<(), Error> {
        self.clipboard.set_text(text)
    }

    #[cfg(feature = "image-data")]
    pub(crate) fn image(self, _image: ImageData<'_>) -> Result<(), Error> {
        Ok(())
    }

    pub(crate) fn html(self, html: Cow<'_, str>, alt: Option<Cow<'_, str>>) -> Result<(), Error> {
        self.clipboard.set_html(html, alt)
    }

    pub(crate) fn file_list(self, _files: &[impl AsRef<std::path::Path>]) -> Result<(), Error> {
        Ok(())
    }
}

pub(crate) struct Clear<'clipboard> {
    _clipboard: &'clipboard mut Clipboard,
}

impl<'clipboard> Clear<'clipboard> {
    pub(crate) fn new(clipboard: &'clipboard mut Clipboard) -> Self {
        Self { _clipboard: clipboard }
    }

    pub(crate) fn clear(self) -> Result<(), Error> {
        Ok(())
    }
}

/// The text of an HTML fragment for the plain-text half of a copy: tags dropped, the common
/// entities decoded. Only used when the caller gave no text alternative.
fn strip_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

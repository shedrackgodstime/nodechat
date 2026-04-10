use anyhow::Context;
use slint::ComponentHandle;
use crate::AppWindow;

pub fn run_app() -> anyhow::Result<()> {
    let app = AppWindow::new().context("failed to create Slint window")?;

    app.on_open_url(|url| {
        #[cfg(target_os = "android")]
        {
            if let Err(e) = open_url_android(url.as_str()) {
                eprintln!("Failed to open URL on Android: {e}");
            }
        }
        #[cfg(not(target_os = "android"))]
        {
            let _ = open::that(url.as_str());
        }
    });

    let handle = app.as_weak();
    app.on_copy_to_clipboard(move |text| {
        if let Some(ui) = handle.upgrade() {
            ui.set_clipboard_buffer(text);
            ui.invoke_do_copy();
        }
    });

    let handle_name = app.as_weak();
    app.on_confirm_name_change(move |name| {
        if let Some(ui) = handle_name.upgrade() {
            ui.set_display_name(name);
        }
    });

    let handle_pass = app.as_weak();
    app.on_submit_password(move |_cur, new_pass| {
        if let Some(ui) = handle_pass.upgrade() {
            ui.set_has_password(!new_pass.is_empty());
        }
    });


    app.run().context("failed to run Slint window")?;
    Ok(())
}

#[cfg(target_os = "android")]
fn open_url_android(url: &str) -> anyhow::Result<()> {
    let ctx = ndk_context::android_context();
    let vm_ptr = ctx.vm().cast::<jni::sys::JavaVM>();
    if vm_ptr.is_null() {
        return Err(anyhow::anyhow!("Android JavaVM pointer is null"));
    }
    let vm = unsafe { jni::JavaVM::from_raw(vm_ptr) }?;
    let mut env = vm.attach_current_thread()?;
    
    let url_str = env.new_string(url)?;
    let uri = env.call_static_method(
        "android/net/Uri",
        "parse",
        "(Ljava/lang/String;)Landroid/net/Uri;",
        &[jni::objects::JValue::from(&url_str)]
    )?.l()?;
    
    let intent_class = env.find_class("android/content/Intent")?;
    let action_view = env.new_string("android.intent.action.VIEW")?;
    let intent = env.new_object(
        &intent_class,
        "(Ljava/lang/String;Landroid/net/Uri;)V",
        &[jni::objects::JValue::from(&action_view), jni::objects::JValue::from(&uri)]
    )?;
    
    let context_ptr = ctx.context() as jni::sys::jobject;
    if context_ptr.is_null() {
        return Err(anyhow::anyhow!("Android Context pointer is null"));
    }
    let activity = unsafe { jni::objects::JObject::from_raw(context_ptr) };
    
    // Add FLAG_ACTIVITY_NEW_TASK so it can open freely from the activity context
    env.call_method(
        &intent,
        "addFlags",
        "(I)Landroid/content/Intent;",
        &[jni::objects::JValue::Int(0x10000000)] // Intent.FLAG_ACTIVITY_NEW_TASK
    )?;

    env.call_method(
        activity,
        "startActivity",
        "(Landroid/content/Intent;)V",
        &[jni::objects::JValue::from(&intent)]
    )?;

    Ok(())
}

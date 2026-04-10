use anyhow::Context;
use slint::ComponentHandle;
use crate::{AppWindow, MockRuntime, Command, ui_models};
use std::thread;

pub fn run_app() -> anyhow::Result<()> {
    let app = AppWindow::new().context("failed to create Slint window")?;
    let runtime = MockRuntime::start();
    let ui_bridge = runtime.ui.clone();

    // --- 1. Event Listener (Backend -> UI) ---
    let ui_handle = app.as_weak();
    let event_bridge = runtime.ui.clone();
    thread::spawn(move || {
        loop {
            // Drain events and apply them to Slint properties
            for event in event_bridge.drain_events() {
                let h = ui_handle.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = h.upgrade() {
                        ui_models::apply_event(&ui, event);
                    }
                });
            }
            // Avoid tight loop
            thread::sleep(std::time::Duration::from_millis(16));
        }
    });

    // --- 2. Callback Wiring (UI -> Backend Commands) ---
    
    // Identity & Global Settings
    let cmd = ui_bridge.clone();
    app.on_confirm_name_change(move |name| {
        let _ = cmd.send(Command::UpdateDisplayName { display_name: name.to_string() });
    });

    let cmd = ui_bridge.clone();
    app.on_submit_password(move |cur, new| {
        let _ = cmd.send(Command::ChangePassword { 
            current_pin: cur.to_string(), 
            new_pin: new.to_string() 
        });
    });

    // Navigation and Context Loading
    let cmd = ui_bridge.clone();
    app.on_load_conversation(move |id, _is_group| {
        let _ = cmd.send(Command::LoadConversation { conversation_id: id.to_string() });
    });

    // Chat Actions
    let cmd = ui_bridge.clone();
    let ui_handle = app.as_weak();
    app.on_send_message(move |text| {
        if let Some(ui) = ui_handle.upgrade() {
            let convo_id = ui.get_active_conversation().id.to_string();
            let _ = cmd.send(Command::SendMessage { 
                conversation_id: convo_id, 
                plaintext: text.to_string() 
            });
        }
    });

    let cmd = ui_bridge.clone();
    app.on_add_peer(move |ticket| {
        let _ = cmd.send(Command::AddContact { ticket_or_peer_id: ticket.to_string() });
    });

    let cmd = ui_bridge.clone();
    let ui_handle = app.as_weak();
    app.on_retry_queued(move || {
        if ui_handle.upgrade().is_some() {
            let _ = cmd.send(Command::Refresh); 
        }
    });

    // Utils & Platform Integration
    app.on_open_url(|url| {
        #[cfg(target_os = "android")]
        { let _ = open_url_android(url.as_str()); }
        #[cfg(not(target_os = "android"))]
        { let _ = open::that(url.as_str()); }
    });

    let cmd = ui_bridge.clone();
    app.on_toggle_verify(move |id, verified| {
        let _ = cmd.send(Command::SetVerification { peer_id: id.to_string(), verified });
    });

    let cmd = ui_bridge.clone();
    app.on_confirm_modal_confirmed(move |slug, pin| {
        match slug.as_str() {
            "clear-history" => { let _ = cmd.send(Command::ClearMessageHistory { scope: crate::contract::HistoryScope::AllConversations, confirmation_pin: Some(pin.to_string()) }); }
            "delete-chat" => { let _ = cmd.send(Command::DeleteConversation { conversation_id: String::new(), confirmation_pin: Some(pin.to_string()) }); } 
            "reset-identity" | "delete-identity" => { let _ = cmd.send(Command::ResetIdentity { confirmation_pin: pin.to_string() }); }
            _ => {}
        }
    });

    let cmd = ui_bridge.clone();
    app.on_toggle_group_candidate(move |id| {
        let _ = cmd.send(Command::ToggleGroupCandidate { contact_id: id.to_string() });
    });

    let cmd = ui_bridge.clone();
    app.on_create_group(move |name, _desc| {
        let _ = cmd.send(Command::CreateGroup { 
            name: name.to_string(), 
            member_contact_ids: Vec::new() // Backend manages this via ToggleGroupCandidate state
        });
    });

    // ... additional UI hooks
    let handle = app.as_weak();
    app.on_copy_to_clipboard(move |text| {
        if let Some(ui) = handle.upgrade() {
            ui.set_clipboard_buffer(text);
            ui.invoke_do_copy();
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

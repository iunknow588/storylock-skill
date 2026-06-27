use super::*;

mod authoring;
mod learning_export;
mod lifecycle;

pub(crate) fn wire_storylock_core_callbacks(
    core: &StoryLockCoreApp,
    package_dir: std::path::PathBuf,
    core_window_slot: Rc<RefCell<Option<StoryLockCoreApp>>>,
    on_closed: Rc<dyn Fn()>,
    host_port: u16,
) {
    let learning_passed = Rc::new(RefCell::new(LearningProgress::new()));
    let answer_editor: Rc<RefCell<Option<AnswerEditorDialog>>> = Rc::new(RefCell::new(None));
    let settings_dialog: Rc<RefCell<Option<StoryLockCoreSettingsDialog>>> =
        Rc::new(RefCell::new(None));

    lifecycle::register_lifecycle_callbacks(
        core,
        &package_dir,
        Rc::clone(&core_window_slot),
        Rc::clone(&on_closed),
        Rc::clone(&settings_dialog),
    );
    authoring::register_authoring_callbacks(
        core,
        &package_dir,
        Rc::clone(&learning_passed),
        Rc::clone(&answer_editor),
    );
    learning_export::register_learning_export_callbacks(
        core,
        &package_dir,
        Rc::clone(&learning_passed),
        host_port,
    );
}

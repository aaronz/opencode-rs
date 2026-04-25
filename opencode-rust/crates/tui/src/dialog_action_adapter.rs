use crate::action::{self, Action, ActionHandler, ActionResult, AppState};
use crate::dialogs::DialogAction as DialogDialogAction;

pub struct DialogActionAdapter;

impl DialogActionAdapter {
    pub fn handle_dialog_action(
        dialog_action: DialogDialogAction,
        state: &mut AppState,
    ) -> ActionResult {
        let action = Self::translate(dialog_action);
        ActionHandler::handle(action, state)
    }

    fn translate(dialog_action: DialogDialogAction) -> Action {
        use DialogDialogAction::*;
        match dialog_action {
            None => Action::Dialog(action::DialogAction::Close),
            Close => Action::Dialog(action::DialogAction::Close),
            Confirm(data) => Action::Dialog(action::DialogAction::ConfirmWithData(data)),
            ConfirmMultiple(items) => {
                Action::Dialog(action::DialogAction::ConfirmWithData(items.join(",")))
            }
            ConfirmModelWithVariant {
                model_id,
                variant_name,
            } => Action::Dialog(action::DialogAction::ConfirmWithData(format!(
                "{}:{}",
                model_id,
                variant_name.unwrap_or_default()
            ))),
            Navigate(direction) => {
                let dialog_action = match direction.as_str() {
                    "up" => action::DialogAction::NavigateUp,
                    "down" => action::DialogAction::NavigateDown,
                    "left" => action::DialogAction::NavigateLeft,
                    "right" => action::DialogAction::NavigateRight,
                    _ => action::DialogAction::Close,
                };
                Action::Dialog(dialog_action)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_close() {
        let dialog_action = DialogDialogAction::Close;
        let action = DialogActionAdapter::translate(dialog_action);
        assert_eq!(action, Action::Dialog(action::DialogAction::Close));
    }

    #[test]
    fn test_translate_confirm_with_data() {
        let dialog_action = DialogDialogAction::Confirm("retry".to_string());
        let action = DialogActionAdapter::translate(dialog_action);
        assert_eq!(
            action,
            Action::Dialog(action::DialogAction::ConfirmWithData("retry".to_string()))
        );
    }

    #[test]
    fn test_translate_confirm_multiple() {
        let dialog_action =
            DialogDialogAction::ConfirmMultiple(vec!["item1".to_string(), "item2".to_string()]);
        let action = DialogActionAdapter::translate(dialog_action);
        assert_eq!(
            action,
            Action::Dialog(action::DialogAction::ConfirmWithData(
                "item1,item2".to_string()
            ))
        );
    }

    #[test]
    fn test_translate_confirm_model_with_variant() {
        let dialog_action = DialogDialogAction::ConfirmModelWithVariant {
            model_id: "gpt-4".to_string(),
            variant_name: Some("turbo".to_string()),
        };
        let action = DialogActionAdapter::translate(dialog_action);
        assert_eq!(
            action,
            Action::Dialog(action::DialogAction::ConfirmWithData(
                "gpt-4:turbo".to_string()
            ))
        );
    }

    #[test]
    fn test_translate_navigate_up() {
        let dialog_action = DialogDialogAction::Navigate("up".to_string());
        let action = DialogActionAdapter::translate(dialog_action);
        assert_eq!(action, Action::Dialog(action::DialogAction::NavigateUp));
    }

    #[test]
    fn test_translate_navigate_down() {
        let dialog_action = DialogDialogAction::Navigate("down".to_string());
        let action = DialogActionAdapter::translate(dialog_action);
        assert_eq!(action, Action::Dialog(action::DialogAction::NavigateDown));
    }

    #[test]
    fn test_translate_navigate_left() {
        let dialog_action = DialogDialogAction::Navigate("left".to_string());
        let action = DialogActionAdapter::translate(dialog_action);
        assert_eq!(action, Action::Dialog(action::DialogAction::NavigateLeft));
    }

    #[test]
    fn test_translate_navigate_right() {
        let dialog_action = DialogDialogAction::Navigate("right".to_string());
        let action = DialogActionAdapter::translate(dialog_action);
        assert_eq!(action, Action::Dialog(action::DialogAction::NavigateRight));
    }

    #[test]
    fn test_translate_navigate_unknown() {
        let dialog_action = DialogDialogAction::Navigate("unknown".to_string());
        let action = DialogActionAdapter::translate(dialog_action);
        assert_eq!(action, Action::Dialog(action::DialogAction::Close));
    }

    #[test]
    fn test_translate_none() {
        let dialog_action = DialogDialogAction::None;
        let action = DialogActionAdapter::translate(dialog_action);
        assert_eq!(action, Action::Dialog(action::DialogAction::Close));
    }

    #[test]
    fn test_handle_dialog_action_updates_state() {
        let mut state = AppState::new();
        state.mode = action::AppMode::ConnectApiKeyError;

        let result =
            DialogActionAdapter::handle_dialog_action(DialogDialogAction::Close, &mut state);

        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.mode, action::AppMode::ConnectApiKey);
    }

    #[test]
    fn test_handle_dialog_action_connect_model_close() {
        let mut state = AppState::new();
        state.mode = action::AppMode::ConnectModel;

        let result =
            DialogActionAdapter::handle_dialog_action(DialogDialogAction::Close, &mut state);

        assert_eq!(result, ActionResult::Handled);
        assert_eq!(state.mode, action::AppMode::ConnectApiKey);
    }
}

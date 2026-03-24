use am::update::AppModel;
use am::ProjectAliases;
use crate::model::TreeNode;

pub fn build_tree(_app_model: &AppModel, _project: Option<&ProjectAliases>) -> Vec<TreeNode> {
    Vec::new()
}

pub fn build_dest_tree(_app_model: &AppModel, _has_project: bool) -> Vec<TreeNode> {
    Vec::new()
}

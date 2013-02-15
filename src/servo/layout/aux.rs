/**
Code for managing the DOM aux pointer
*/

use dom::node::{AbstractNode, LayoutData};
use core::dvec::DVec;

pub trait LayoutAuxMethods {
    fn initialize_layout_data(self) -> Option<@LayoutData>;
    fn initialize_style_for_subtree(self, refs: &DVec<@LayoutData>);
}

impl AbstractNode : LayoutAuxMethods {
    /// If none exists, creates empty layout data for the node (the reader-auxiliary
    /// box in the COW model) and populates it with an empty style object.
    fn initialize_layout_data(self) -> Option<@LayoutData> {
        let node = self.as_node();
        match node.layout_data {
            Some(_) => None,
            None => {
                let data = Some(LayoutData::new());
                node.data = data;
                data
            }
        }
    }

    /// Initializes layout data and styles for a Node tree, if any nodes do not have
    /// this data already. Append created layout data to the task's GC roots.
    fn initialize_style_for_subtree(self, refs: &DVec<@LayoutData>) {
        do self.traverse_preorder |n| {
            match n.initialize_layout_data() {
                Some(r) => refs.push(r),
                None => {}
            }
        }
    }

}

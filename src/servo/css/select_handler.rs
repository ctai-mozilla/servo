///
/// Implementation of the callbacks that the CSS selector engine uses to query the DOM.
///

use dom::node::AbstractNode;
use newcss::select::SelectHandler;

pub struct NodeSelectHandler {
    node: AbstractNode
}

fn with_node_name<R>(node: AbstractNode, f: &fn(&str) -> R) -> R {
    if !node.is_element() {
        fail!(~"attempting to style non-element node");
    }
    f(node.as_element().tag_name)
}

impl SelectHandler<AbstractNode> for NodeSelectHandler {
    fn with_node_name<R>(node: &AbstractNode, f: &fn(&str) -> R) -> R {
        with_node_name(*node, f)
    }

    fn named_parent_node(node: &AbstractNode, name: &str) -> Option<AbstractNode> {
        match node.parent() {
            Some(parent) => {
                do with_node_name(parent) |node_name| {
                    if name == node_name {
                        Some(parent)
                    } else {
                        None
                    }
                }
            }
            None => None
        }
    }

    fn parent_node(node: &AbstractNode) -> Option<AbstractNode> {
        node.parent()
    }

    // TODO: Use a Bloom filter.
    fn named_ancestor_node(node: &AbstractNode, name: &str) -> Option<AbstractNode> {
        let mut node = *node;
        loop {
            let parent = node.parent();
            match parent {
                Some(parent) => {
                    let mut found = false;
                    do with_node_name(parent) |node_name| {
                        if name == node_name {
                            found = true;
                        }
                    }
                    if found {
                        return Some(parent);
                    }
                    node = parent;
                }
                None => return None
            }
        }
    }

    fn node_is_root(node: &AbstractNode) -> bool {
        self.parent_node(node).is_none()
    }

    fn with_node_id<R>(node: &AbstractNode, f: &fn(Option<&str>) -> R) -> R {
        if !node.is_element() {
            fail!(~"attempting to style non-element node");
        }
        f(node.as_element().get_attr("id"))
    }

    fn node_has_id(node: &AbstractNode, id: &str) -> bool {
        if !node.is_element() {
            fail!(~"attempting to style non-element node");
        }
        match node.as_element().get_attr("id") {
            None => false,
            Some(existing_id) => id == existing_id
        }
    }
}

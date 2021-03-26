#[derive(PartialEq, Eq, Debug)]
pub struct ListNode {
    pub val: i32,
    pub next: Option<Box<ListNode>>,
}

impl ListNode {
    #[inline]
    pub fn new(val: i32) -> Self {
        ListNode { next: None, val }
    }
}

// helper function for test
pub fn to_list(vec: Vec<i32>) -> Option<Box<ListNode>> {
    let mut current = None;
    for &v in vec.iter().rev() {
        let mut node = ListNode::new(v);
        node.next = current;
        current = Some(Box::new(node));
    }
    current
}

pub fn to_list_recur(vec: &Vec<i32>) -> Option<Box<ListNode>> {
    if let Some((first, tail)) = vec.split_first() {
        let mut head = ListNode::new(first.clone());
        head.next = to_list_recur(&Vec::from(tail));
        Some(Box::new(head))
    } else {
        None
    }
}

#[macro_export]
macro_rules! linked {
    ($($e:expr),*) => {to_list(vec![$($e.to_owned()), *])};
    ($($e:expr,)*) => {to_list(vec![$($e.to_owned()), *])};
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link() {
        let vec1 = vec!{1, 2, 3, 45};
        println!("list: {:?}", to_list_recur(&vec1));
    }
}

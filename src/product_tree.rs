use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct ProductTree<T> {
    tree: Vec<Vec<T>>,
    tree_depth: usize,
    input_len: usize,
}

impl<T> ProductTree<T>
where
    T: Sync + Send + Clone,
{
    pub fn new(
        input: Vec<T>,
        product_op: impl Fn(&T, &T) -> T + Sync + Send,
        comp_root: bool,
    ) -> Self {
        let len = input.len();
        // equivalent to len.log2_ceil()
        let mut tree_depth = (usize::BITS - 1 - len.next_power_of_two().leading_zeros()) as usize;
        if comp_root {
            tree_depth += 1;
        }
        let mut product_tree = Vec::with_capacity(tree_depth);
        product_tree.push(input);
        for i in 0..tree_depth - 1 {
            let layer = product_tree[i]
                .par_iter()
                .chunks(2)
                .map(|chunk| {
                    if chunk.len() == 2 {
                        product_op(chunk[0], chunk[1])
                    } else {
                        chunk[0].clone()
                    }
                })
                .collect();
            product_tree.push(layer);
        }
        debug_assert_eq!(tree_depth, product_tree.len());

        Self {
            tree: product_tree,
            tree_depth,
            input_len: len,
        }
    }

    pub fn all_products(
        &self,
        identity_op: impl Fn() -> T + Sync + Send,
        product_op: impl Fn(&T, &T) -> T + Sync + Send,
    ) -> Vec<T> {
        let mut ans = Vec::with_capacity(self.input_len);
        (0..self.input_len)
            .into_par_iter()
            .map(|mut i| {
                let mut i_bits = Vec::with_capacity(self.tree_depth);
                for _ in 0..self.tree_depth {
                    i_bits.push(i % 2);
                    i >>= 1;
                }

                let mut v = identity_op();
                let mut index = 0;
                for (depth, bit) in i_bits.into_iter().enumerate().rev() {
                    index = 2 * index + bit;
                    let neighbor_index = if bit == 0 { index + 1 } else { index - 1 };
                    let layer = &self.tree[depth];
                    if neighbor_index < layer.len() {
                        v = product_op(&v, &layer[neighbor_index]);
                    }
                }
                v
            })
            .collect_into_vec(&mut ans);
        ans
    }

    pub fn root(mut self) -> T {
        let mut root = self.tree.pop().unwrap();
        debug_assert_eq!(root.len(), 1);
        root.pop().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let product_op = |a: &i32, b: &i32| -> i32 { a * b };
        let identity_op = || -> i32 { 1 };
        let input = vec![1, 2, 3, 4, 5];
        let product_tree1 = ProductTree::new(input.clone(), product_op, true);
        let product_tree2 = ProductTree::new(input, product_op, false);
        let all_products1 = product_tree1.all_products(identity_op, product_op);
        let all_products2 = product_tree2.all_products(identity_op, product_op);
        assert_eq!(all_products1, all_products2);
        assert_eq!(product_tree1.root(), 120);
    }
}

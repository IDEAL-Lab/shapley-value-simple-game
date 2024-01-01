use rayon::prelude::*;

#[derive(Clone)]
pub struct Union<T> {
    max_id: usize,
    data: T,
}

impl<T> Union<T> {
    pub fn get(&self) -> &T {
        &self.data
    }

    pub fn into_inner(self) -> T {
        self.data
    }
}

pub struct UnionCombination<T>(pub Vec<Union<T>>);

impl<T> UnionCombination<T>
where
    T: Sync + Send + Clone,
{
    pub fn new<INIT, INC>(input_len: usize, init_op: INIT, inc_op: INC) -> Self
    where
        INIT: Fn(usize) -> T + Sync + Send,
        INC: Fn(&T, usize) -> Option<T> + Sync + Send,
    {
        let mut cur = 0;
        let mut unions: Vec<Union<T>> = (0..input_len)
            .into_par_iter()
            .map(|id| Union {
                max_id: id,
                data: init_op(id),
            })
            .collect();

        while cur < unions.len() {
            let new_unions: Vec<Union<T>> = unions[cur..]
                .par_iter()
                .flat_map(|old_u| {
                    (old_u.max_id + 1..input_len)
                        .into_par_iter()
                        .filter_map(|new_id| {
                            let data = inc_op(&old_u.data, new_id)?;
                            Some(Union {
                                max_id: new_id,
                                data,
                            })
                        })
                })
                .collect();

            cur = unions.len();
            unions.extend(new_unions);
        }

        Self(unions)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

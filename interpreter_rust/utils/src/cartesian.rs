use std::iter::repeat_n;

pub fn cartesian<T: Clone>(xss: Vec<Vec<T>>, ys: Vec<T>) -> Vec<Vec<T>> {
    let mut zss = Vec::with_capacity(xss.len() * ys.len());
    for (ys, mut xs) in repeat_n(ys, xss.len()).zip(xss) {
        xs.reserve(1);
        for (mut xs, y) in repeat_n(xs, ys.len()).zip(ys) {
            xs.push(y);
            zss.push(xs);
        }
    }

    zss
}

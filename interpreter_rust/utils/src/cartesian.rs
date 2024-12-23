#[expect(clippy::needless_pass_by_value)]
pub fn cartesian<T: Clone>(xss: Vec<Vec<T>>, ys: Vec<T>) -> Vec<Vec<T>> {
    xss.into_iter()
        .flat_map(|xs| {
            ys.iter().cloned().map(move |y| {
                let mut xs = xs.clone();
                xs.push(y);
                xs
            })
        })
        .collect()
}

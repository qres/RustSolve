#![allow(non_snake_case)]

extern crate ndarray;

use marke::{Linear, Symmetric};

pub mod stencil;
pub mod jacobi;
pub mod multigrid;
pub mod marke;

/* IDEA
Solver::new(size)
.cg()
.take(max_iters)
.take_while(r >= 1e-10)
.inspect(println)
.restart(10)
.take_while(r >= 1e-10)
.inspect(println)
.solve()
*/

use std::cmp;

pub trait Vector {
    fn dot(&self, other: &Self) -> f64;
    fn norm2_sq(&self) -> f64 {
        self.dot(&self)
    }
    fn norm_max(&self) -> f64;
    fn set_copy(&mut self, src: &Self);
    fn set_ax(&mut self, a: f64, src: &Self);
    fn set_axpy(&mut self, a: f64, x: &Self, y: &Self);
    fn inc_ax(&mut self, a: f64, x: &Self) {
        self.acc_mul_bx(1.0, a, x);
    }
    /// self += a*x + y
    fn inc_axpy(&mut self, a: f64, x: &Self, y: &Self) {
        self.inc_axpby(a, x, 1.0, y);
    }
    fn inc_axpby(&mut self, a: f64, x: &Self, b: f64, y: &Self);
    /// self = a*self + b*x
    fn acc_mul_bx(&mut self, a: f64, b: f64, x: &Self);
    fn acc_mul_bxpy(&mut self, a: f64, b: f64, x: &Self, y: &Self);
}

pub trait Matrix<V> {
    fn mul(out: &mut V, mat: &Self, rhs: &V);
    fn aAxpby(out: &mut V, a: f64, A: &Self, x: &V, b: f64, y: &V);
    fn inc_aAxpby(out: &mut V, a: f64, A: &Self, x: &V, b: f64, y: &V);
}

impl<'m,V,M> Matrix<V> for &'m M where M: Matrix<V> {
    fn mul(out: &mut V, mat: &Self, rhs: &V) {
        M::mul(out, mat, rhs);
    }
    fn aAxpby(out: &mut V, a: f64, A: &Self, x: &V, b: f64, y: &V) {
        M::aAxpby(out, a, A, x, b, y);
    }
    fn inc_aAxpby(out: &mut V, a: f64, A: &Self, x: &V, b: f64, y: &V) {
        M::inc_aAxpby(out, a, A, x, b, y);
    }
}

fn get<F>(f: F) where F: Fn(u32, f64) {

}

fn get2() {
    get(black_hole2);
    get(|_,_| {});
}

// not doing anything, used as defult for the inspect methods
fn black_hole2<A,B>(_: A, _: B) { }

mod nd {
    use super::*;

    use ndarray::{Array1, Array2, ArrayView1, Zip};

    impl Vector for Array1<f64> {
        fn dot(&self, other: &Self) -> f64 {
            Array1::<f64>::dot(&self, other)
        }

        fn norm_max(&self) -> f64 {
            self.fold(0.0, |acc, elem| {
                acc.max(elem.abs())
            })
        }

        fn set_copy(&mut self, src: &Self) {
            *self = src.clone()
        }

        fn set_ax(&mut self, a: f64, src: &Self) {
            *self = a * src
        }

        fn set_axpy(&mut self, a: f64, x: &Self, y: &Self) {
            Zip::from(self).and(x).and(y).apply(|s,x,y| {
                *s = a * x + y
            });
        }

        fn inc_axpby(&mut self, a: f64, x: &Self, b: f64, y: &Self) {
            Zip::from(self).and(x).and(y).apply(|s,x,y| {
                *s += a * x + b * y
            });
        }

        fn acc_mul_bx(&mut self, a: f64, b: f64, x: &Self) {
            Zip::from(self).and(x).apply(|acc,x| {
                *acc = a * *acc + b * x
            });
        }

        fn acc_mul_bxpy(&mut self, a: f64, b: f64, x: &Self, y: &Self) {
            Zip::from(self).and(x).and(y).apply(|acc,x,y| {
                *acc = a * *acc + b * x + y
            });
        }

    }

    impl Vector for Array2<f64> {
        fn dot(&self, other: &Self) -> f64 {
            let mut sum = 0.0;
            for i in 0..self.shape()[0] {
                for j in 0..self.shape()[1] {
                    sum += self[[i,j]] * other[[i,j]]
                }
            }
            sum
        }

        fn norm_max(&self) -> f64 {
            self.fold(0.0, |acc, elem| {
                acc.max(elem.abs())
            })
        }

        fn set_copy(&mut self, src: &Self) {
            *self = src.clone()
        }

        fn set_ax(&mut self, a: f64, src: &Self) {
            *self = a * src
        }

        fn set_axpy(&mut self, a: f64, x: &Self, y: &Self) {
            Zip::from(self).and(x).and(y).apply(|s,x,y| {
                *s = a * x + y
            });
        }

        fn inc_axpby(&mut self, a: f64, x: &Self, b: f64, y: &Self) {
            Zip::from(self).and(x).and(y).apply(|s,x,y| {
                *s += a * x + b * y
            });
        }

        fn acc_mul_bx(&mut self, a: f64, b: f64, x: &Self) {
            Zip::from(self).and(x).apply(|acc,x| {
                *acc = a * *acc + b * x
            });
        }

        fn acc_mul_bxpy(&mut self, a: f64, b: f64, x: &Self, y: &Self) {
            Zip::from(self).and(x).and(y).apply(|acc,x,y| {
                *acc = a * *acc + b * x + y
            });
        }

    }

    impl Matrix<Array1<f64>> for Array2<f64> {
        fn mul(out: &mut Array1<f64>, A: &Array2<f64>, rhs: &Array1<f64>) {
            *out = A.dot(rhs)
        }

        fn aAxpby(out: &mut Array1<f64>, a: f64, A: &Array2<f64>, x: &Array1<f64>, b: f64, y: &Array1<f64>) {
            *out = a * A.dot(x) + b * y;
        }

        fn inc_aAxpby(out: &mut Array1<f64>, a: f64, A: &Array2<f64>, x: &Array1<f64>, b: f64, y: &Array1<f64>) {
            *out += &(a * A.dot(x) + b * y);
        }
    }

    impl Preconditioner<Array1<f64>> for Array2<f64> {
        type A = [Array1<f64>];

        fn size(&self) -> usize {
            0
        }

        fn apply(&self, out: &mut Array1<f64>, x: &Array1<f64>, _:&mut [Array1<f64>]) {
            *out = self.dot(x);
        }

    }

    impl<'a> Preconditioner<Array1<f64>> for ArrayView1<'a, f64> {
        type A = [Array1<f64>];

        fn size(&self) -> usize {
            0
        }

        fn apply(&self, out: &mut Array1<f64>, x: &Array1<f64>, _:&mut [Array1<f64>]) {
            Zip::from(out).and(self).and(x).apply(|out,s,x| {
                *out = s * x
            });
        }

    }
}


// TODO: I don't like this
pub struct Solver<M,V,S> {
    solver: S,
    vectors: Vec<V>,
    booh: std::marker::PhantomData<M>,
}

impl<M,V,S> Solver<M,V,S> where V: Vector, M: Matrix<V>, S: SolverImpl<M,V,A=[V]> {
    pub fn new(vecs: Vec<V>, solver: S) -> Self {
        // we implicitly assume, that the size of the individual vectors is correct TODO
        assert!(vecs.len() >= solver.size());
        Solver {
            vectors: vecs, // TODO pre allocate
            solver: solver,
            booh: std::marker::PhantomData,
        }
    }

    pub fn solve(&mut self, A: &M, x: &mut V, b: &V) {
        self.solver.solve(A, x, b, &mut self.vectors);
    }
}

pub trait SolverImpl<M,V> {
    type A: ?Sized;
    fn size(&self) -> usize;
    fn solve(&self, A: &M, x: &mut V, b: &V, vecs: &mut Self::A);
}

pub trait Preconditioner<V> {
    type A: ?Sized;
    fn size(&self) -> usize;
    fn apply(&self, out: &mut V, x: &V, vecs: &mut Self::A);
}

pub trait BreakCondition {
    fn continue_iter(&self, res: f64) -> bool;
}

pub struct Absolute(f64);
pub struct NoResCeck();

impl BreakCondition for Absolute {
    fn continue_iter(&self, res: f64) -> bool {
        res > self.0
    }
}

impl BreakCondition for NoResCeck {
    fn continue_iter(&self, _: f64) -> bool {
        true
    }
}

pub struct DiagSolver;

pub struct Id;
impl Symmetric for Id {}
impl Linear for Id {}

impl Id {
    pub fn new() -> Self {
        Id{}
    }
}

impl<V> SolverImpl<Id,V> for DiagSolver where V: Vector {
    type A = [V];
    fn size(&self) -> usize {
        0
    }

    fn solve(&self, _: &Id, x: &mut V, b: &V, _: &mut [V]) {
        // nothing to do
        x.set_copy(b);
    }
}

impl<V> Preconditioner<V> for Id where V: Vector {
    type A = [V];
    fn size(&self) -> usize {
        0
    }

    fn apply(&self, out: &mut V, x: &V, _: &mut [V]) {
        out.set_copy(x);
    }
}

/// constant
impl Symmetric for f64 {}
impl Linear for f64 {}

impl<V> SolverImpl<f64,V> for DiagSolver where V: Vector {
    type A = [V];
    fn size(&self) -> usize {
        0
    }

    fn solve(&self, scalar: &f64, x: &mut V, b: &V, _: &mut [V]) {
        // oh, thats easy! ;)
        //// x = 1 / scalar * b
        x.acc_mul_bx(0.0, 1.0 / scalar, b);
    }
}

impl<V> Preconditioner<V> for f64 where V: Vector {
    type A = [V];
    fn size(&self) -> usize {
        0
    }

    fn apply(&self, out: &mut V, x: &V, _: &mut [V]) {
        out.set_ax(1.0 / *self, x); // TODO this schould be *self only XXX
    }
}

/// Applies `A^-1` as preconditioner using a solver to solve `A^-1 x`
pub struct InvMatrix<M, S> {
    A: M,
    solver: S,
}
impl<M,S> Symmetric for InvMatrix<M,S> where S: Symmetric, M: Symmetric {}
impl<M,S> Linear for InvMatrix<M,S> where S: Linear, M: Linear {}

impl<V, M, S> Preconditioner<V> for InvMatrix<M, S> where M: Matrix<V>, S: SolverImpl<M,V,A=[V]> {
    type A = [V];
    fn size(&self) -> usize {
        self.solver.size()
    }

    fn apply(&self, out: &mut V, x: &V, vecs: &mut [V]) {
        //// out = A^-1 x
        self.solver.solve(&self.A, out, x, vecs)
    }
}


/// Using the Neumann Series to approximate the inverse of a Matrix.
/// This is equivalent to a Richardson iteration.
///
/// ```text
/// A = I - B such that ||B|| = ||I-A|| < 1 for some norm ||.||
///
/// M_inv ~= A_inv = sum (I - A)^m = I + (I - A) + (I-A)^2 + ...
/// Using Horner Scheme
/// m=0: M_0_inv = I
/// m=1: M_1_inv = I + (I-A)           = I + (I-A)M_0_inv
/// m=2: M_2_inv = I + (I-A) + (I-A)^2 = I + (I-A)M_1_inv
/// m=k: M_k_inv = I +  ...  + (I-A)^k = I + (I-A)M_k-1_inv
/// ```
pub struct NeumannSeries<M> {
    order: usize,
    scale: f64,
    A: M,
}
impl<M> Symmetric for NeumannSeries<M> where M: Symmetric {}
impl<M> Linear for NeumannSeries<M> where M: Linear {}

impl<M> NeumannSeries<M> {
    pub fn new(order: usize, A: M) -> Self {
        NeumannSeries {
            order: order,
            scale: 1.0,
            A: A,
        }
    }

    pub fn scale(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }
}

impl<V,M> Preconditioner<V> for NeumannSeries<M> where V: Vector, M: Matrix<V> {
    type A = [V];
    fn size(&self) -> usize {
        1
    }

    fn apply(&self, out: &mut V, x: &V, vecs: &mut [V]) {
        if self.order > 0 {
            let mut h = out;
            let mut z = vecs.iter_mut().next().expect("vector buffer too small");
            //// h = x
            h.set_copy(x);
            // first iterations
            // we use s*A insted of A, i.e. we approximate (sA)^-1 = 1/s A^-1
            // => s(sA)^-1 = A^-1
            for _ in 1..self.order {
                //// h = h + x - s A h
                //          ^~  =:z  ~^
                M::aAxpby(&mut z, -self.scale, &self.A, &h, 1.0, x);
                h.inc_ax(1.0, z);
            }
            // last iteration
            //// h = s (h + x - s A h)
            ////           ^~  =:z  ~^
            M::aAxpby(&mut z, -self.scale, &self.A, &h, 1.0, x);
            // undo scaling: A^-1 = s(sA)^-1
            h.acc_mul_bx(self.scale, self.scale, z);
        } else {
            out.set_copy(x);
        }
    }
}

pub struct Restart<S>{
    count: u32,
    solver: S,
}
impl<S> Symmetric for Restart<S> where S: Symmetric {}
impl<S> Linear for Restart<S> where S: Linear {}

impl<S> Restart<S> {
    pub fn new(count: u32, solver: S) -> Self {
        Restart {
            count: count,
            solver: solver,
        }
    }
}

impl<M,V,S> SolverImpl<M,V> for Restart<S> where M: Matrix<V>, V: Vector, S: SolverImpl<M,V,A=[V]> {
    type A = [V];
    fn size(&self) -> usize {
        0
    }

    fn solve(&self, A: &M, x: &mut V, b: &V, vecs: &mut [V]) {
        for _ in 0..self.count {
            self.solver.solve(A, x, b, vecs);
        }
    }
}

pub struct Concat<S1, S2> {
    solver1: S1,
    solver2: S2,
}
impl<S1,S2> Symmetric for Concat<S1,S2> where S1: Symmetric, S2: Symmetric {}
impl<S1,S2> Linear for Concat<S1,S2> where S1: Linear, S2: Linear {}

impl<S1,S2> Concat<S1,S2> {

}

impl<M,V,S1,S2> SolverImpl<M,V> for Concat<S1,S2> where M: Matrix<V>, V: Vector, S1: SolverImpl<M,V,A=[V]>, S2: SolverImpl<M,V,A=[V]> {
    type A = [V];
    fn size(&self) -> usize {
        let s1 = self.solver1.size();
        let s2 = self.solver2.size();
        cmp::max(s1, s2)
    }

    fn solve(&self, A: &M, x: &mut V, b: &V, vecs: &mut [V]) {
        self.solver1.solve(A, x, b, vecs);
        self.solver2.solve(A, x, b, vecs);
    }
}

pub struct CG<F,FR> where F: BreakCondition {
    max_iters: u32,
    per_iter: F,
    inspect_res: FR,
}
impl<FR> Linear for CG<NoResCeck,FR> {}

// we split the implementation to improve type inferrence
impl<F> CG<F, fn(u32, f64)> where F: BreakCondition,  {
    pub fn new(max_iters: u32, per_iter: F) -> Self {
        CG {
            max_iters: max_iters,
            per_iter: per_iter,
            inspect_res: black_hole2 as fn(u32,f64),
        }
    }
}
impl<F,FR> CG<F, FR> where F: BreakCondition {
    pub fn inspect_residual<FRNEW>(self, f: FRNEW) -> CG<F,FRNEW> where FRNEW: Fn(u32, f64){
        CG {
            max_iters: self.max_iters,
            per_iter: self.per_iter,
            inspect_res: f,
        }
    }
}

impl<M,V,F,FR> SolverImpl<M,V> for CG<F,FR> where M: Matrix<V>, V: Vector, F: BreakCondition, FR: Fn(u32, f64) {
    type A = [V];
    fn size(&self) -> usize {
        3
    }

    fn solve(&self, A: &M, x: &mut V, b: &V, vecs: &mut [V]) {
        // TODO slice pattern
        let mut vecs = vecs.iter_mut();
        let mut r = vecs.next().expect("vector buffer too small");
        let mut p = vecs.next().expect("vector buffer too small");
        let mut v = vecs.next().expect("vector buffer too small");

        //// r = b - A x
        M::aAxpby(&mut r, -1.0, &A, &x, 1.0, &b);
        //// p = r
        p.set_copy(&r);
        //// a_k = ||r||^2
        let mut alpha = r.norm2_sq();
        //// a_0 = ||r||^2
        let alpha0 = alpha;

        (self.inspect_res)(0, alpha.sqrt());

        if !self.per_iter.continue_iter(r.norm2_sq().sqrt()) || alpha0 == 0.0 { // for the case that per_iter(.) == true
            return;
        }

        'iteration: for k in 0..self.max_iters {
            //// v = A p
            M::mul(&mut v, &A, &p);
            //// lambda = a / <v,p>
            let lambda = alpha / v.dot(&p);
            //// x += lambda p
            x.inc_ax(lambda, &p);
            //// r -= lambda v
            r.inc_ax(-lambda, &v);
            //// a_k+1 = ||r||^2
            let alpha_kp1 = r.norm2_sq();

            (self.inspect_res)(k+1, alpha_kp1.sqrt());
            if !self.per_iter.continue_iter(alpha_kp1.sqrt()) || alpha_kp1 == 0.0 {
                break 'iteration;
            }
            //// p = a_k+1 / a * p + r
            p.acc_mul_bx(alpha_kp1 / alpha, 1.0, &r);
            //// a_k = a_k+1
            alpha = alpha_kp1;
        }

    }
}

pub struct PCG<F,P,FR> where F: BreakCondition {
    cg: CG<F,FR>,
    preconditioner: P,
}
impl<P,FR> Linear for PCG<NoResCeck,P,FR> where P: Linear {}

impl<F,P,FR> PCG<F,P,FR> where F: BreakCondition {
    pub fn new(p: P, cg: CG<F,FR>) -> Self {
        PCG {
            cg: cg,
            preconditioner: p,
        }
    }
}

impl<M,V,F,P,FR> SolverImpl<M,V> for PCG<F,P,FR> where M: Matrix<V>, V: Vector, F: BreakCondition, P: Preconditioner<V,A=[V]>, FR: Fn(u32, f64) {
    type A = [V];
    fn size(&self) -> usize {
        let pc_hint = self.preconditioner.size();
        let pcg_min = 4;
        pcg_min + pc_hint
    }

    fn solve(&self, A: &M, x: &mut V, b: &V, vecs: &mut [V]) {
        let (mut vecs, mut rest) = vecs.split_at_mut(4);
        let mut vecs = vecs.iter_mut();
        let mut r = vecs.next().expect("vector buffer too small");
        let mut p = vecs.next().expect("vector buffer too small");
        let mut v = vecs.next().expect("vector buffer too small");
        let mut z = vecs.next().expect("vector buffer too small");

        //// r = b - A x
        M::aAxpby(&mut r, -1.0, &A, &x, 1.0, &b);
        //// p = P^-1 r
        self.preconditioner.apply(&mut p, &r, &mut rest);
        //// a = <r,p>
        let mut alpha = r.dot(&p);
        let alpha0 = alpha;

        let r_norm = r.norm2_sq().sqrt(); // TODO will this be removed if we dont inspect/check the residual
        (self.cg.inspect_res)(0, r_norm);
        if !self.cg.per_iter.continue_iter(r_norm) || alpha0 == 0.0 { // for the case that per_iter(.) == true
            return;
        }

        'iteration: for k in 0..self.cg.max_iters {
            //// v = A p
            M::mul(&mut v, &A, &p);
            //// lambda = a / <v,p>
            let lambda = alpha / v.dot(&p);
            //// x += lambda p
            x.inc_ax(lambda, &p);
            //// r -= lambda v
            r.inc_ax(-lambda, &v);

            let r_norm = r.norm2_sq().sqrt(); // TODO will this be removed if we dont inspect/check the residual
            (self.cg.inspect_res)(k+1, r_norm);
            if !self.cg.per_iter.continue_iter(r_norm) { // we cannot use a_k+1 for convergence checks
                break 'iteration;
            }
            //// z = P^-1 r
            self.preconditioner.apply(&mut z, &r, &mut rest); // TODO reuse v as z
            //// a_k+1 = <r,z>
            let alpha_kp1 = r.dot(&z);
            //// p = a_k+1 / a * p + z
            p.acc_mul_bx(alpha_kp1 / alpha, 1.0, &z);
            //// a_k = a_k+1
            alpha = alpha_kp1;
            //v = z; // end reuse v as z
        }

    }
}

pub struct BiCGStab<F,FR> where F: BreakCondition {
    max_iters: u32,
    per_iter: F,
    inspect_res: FR,
}
// not Linear as there exist critical conditions

impl<F> BiCGStab<F,fn(u32,f64)> where F: BreakCondition {
    pub fn new(max_iters: u32, per_iter: F) -> Self {
        BiCGStab {
            max_iters: max_iters,
            per_iter: per_iter,
            inspect_res: black_hole2 as _,
        }
    }
}
impl<F,FR> BiCGStab<F,FR> where F: BreakCondition {
    pub fn inspect_residual<FRNEW>(self, f: FRNEW) -> BiCGStab<F,FRNEW> where FRNEW: Fn(u32, f64){
        BiCGStab {
            max_iters: self.max_iters,
            per_iter: self.per_iter,
            inspect_res: f,
        }
    }
}

impl<M,V,F,FR> SolverImpl<M,V> for BiCGStab<F,FR> where M: Matrix<V>, V: Vector, F: BreakCondition, FR: Fn(u32,f64) {
    type A = [V];
    fn size(&self) -> usize {
        6
    }

    fn solve(&self, A: &M, x: &mut V, b: &V, vecs: &mut [V]) {
        let mut vecs = vecs.iter_mut();
        let mut r = vecs.next().expect("vector buffer too small");
        let mut r0 = vecs.next().expect("vector buffer too small");
        let mut p = vecs.next().expect("vector buffer too small");
        let mut v = vecs.next().expect("vector buffer too small");
        let mut s = vecs.next().expect("vector buffer too small");
        let mut t = vecs.next().expect("vector buffer too small");

        //// r = b - A x
        M::aAxpby(&mut r, -1.0, A, x, 1.0, b);
        //// r_0 = r
        r0.set_copy(&r);
        //// p = r
        p.set_copy(&r);
        //// rho = ||r||^2
        let mut rho = r.norm2_sq();

        (self.inspect_res)(0, rho.sqrt());
        if !self.per_iter.continue_iter(rho.sqrt()) || rho == 0.0 { // for the case that per_iter(.) == true
            return;
        }


        'iteration: for k in 0..self.max_iters {
            //// v = A p
            M::mul(&mut v, A, &p);
            //// alpha = rho / <v, r_0>
            let alpha = rho / v.dot(&r0);

            //// s = r - alpha * v
            s.set_axpy(-alpha, &v, r);
            //// t = A s
            M::mul(&mut t, A, &s);
            //// omega = <t,s> / ||t||^2
            let omega = t.dot(&s) / t.norm2_sq();
            //// x += alpha * p + omega * s
            x.inc_axpby(alpha, &p, omega, &s);
            //// r = s - omega * t
            r.set_axpy(-omega, &t, &s);

            let r_norm = r.norm2_sq().sqrt();
            (self.inspect_res)(k+1, r_norm);
            if !self.per_iter.continue_iter(r_norm) { // TODO  || rho == 0.0 ?
                break 'iteration;
            }
            //// rho_k+1 = <r,r0>
            let rho_kp1 = r.dot(&r0);
            //// beta = alpha * rho_kp1 / (omega * rhp)
            let beta = alpha * rho_kp1 / (omega * rho);
            //// p = beta * (p - omega * v) + r
            p.acc_mul_bxpy(beta, -beta * omega, &v, &r);
            //// rho = rho_kp1
            rho = rho_kp1;
        }
    }
}

pub type BiCGStabM<F,FR> = Restart<BiCGStab<F,FR>>;

#[cfg(test)]
mod test {
    use super::*;
    use ndarray::{Array1, Array2};
    use super::{Solver, SolverImpl};
    use std::f64::consts::PI;

    const N: u32 = 101;

    fn test_convergence<S>(solver: &mut Solver<Array2<f64>,Array1<f64>,S>) where S: SolverImpl<Array2<f64>,Array1<f64>,A=[Array1<f64>]> {
        use ndarray::Dim;
        let h = 1.0/(N-1) as f64;
        let A = -1.0/h/h*Array2::from_shape_fn(Dim([N as usize, N as usize]), |(i,j)| {
            if i==j {
                return -2.0
            }
            if i+1==j {
                return 1.0
            }
            if i==j+1 {
                return 1.0
            }
            return 0.0
        });
        let mut x0 = Array1::from_elem(N as usize, 0.0);
        let x_star = Array1::from_shape_fn(N as usize, |i| {
            (i as f64 * h * PI).sin()
        });
        // we want to know how good our solvers are and not the discretisation
        let b = A.dot(&x_star);

        solver.solve(&A, &mut x0, &b);

        println!("ERROR {:.3e}", (&x_star - &x0).norm_max());
        assert!((x_star - x0).norm_max() < 1e-13);
    }

    #[test]
    fn solver_cg() {
        let vecs = vec![Array1::from_elem(N as usize, 0.0); 10];

        let mut solver = Solver::new(
            vecs,
            CG::new(N, Absolute(1e-12))
                .inspect_residual(|k,r| {println!("k: {:>3} r: {:.3e}",k,r)})
        );
        test_convergence(&mut solver);
    }

    #[test]
    fn solver_pcg_id() {
        let vecs = vec![Array1::from_elem(N as usize, 0.0); 10];

        let mut solver = Solver::new(
            vecs,
            PCG::new(
                Id::new(),
                CG::new(N, Absolute(1e-12)),
            )
        );
        test_convergence(&mut solver);
    }

    #[test]
    fn solver_pcg_jac() {
        let vecs = vec![Array1::from_elem(N as usize, 0.0); 10];

        use ndarray::Dim;
        let h = 1.0/(N-1) as f64;
        let A = -1.0/h/h*Array2::from_shape_fn(Dim([N as usize, N as usize]), |(i,j)| {
            if i==j {
                return -2.0
            }
            if i+1==j {
                return 1.0
            }
            if i==j+1 {
                return 1.0
            }
            return 0.0
        });
        let mut x0 = Array1::from_elem(N as usize, 0.0);
        let x_star = Array1::from_shape_fn(N as usize, |i| {
            (i as f64 * h * PI).sin()
        });
        // we want to know how good our solvers are and not the discretisation
        let b = A.dot(&x_star);

        let mut solver = Solver::new(
            vecs,
            PCG::new(
                A.diag(), // TODO
                CG::new(N, NoResCeck{}),
            )
        );
        solver.solve(&A, &mut x0, &b);

        println!("ERROR {:.3e}", (&x_star - &x0).norm_max());
        assert!((x_star - x0).norm_max() < 1e-13);
    }

    #[test]
    fn solver_pcg_neumann() {
        let vecs = vec![Array1::from_elem(N as usize, 0.0); 10];

        use ndarray::Dim;
        let h = 1.0/(N-1) as f64;
        let A = -1.0/h/h*Array2::<f64>::from_shape_fn(Dim([N as usize, N as usize]), |(i,j)| {
            if i==j {
                // make it strictly diagonal dominant
                return -4.0
            }
            if i+1==j {
                return 1.0
            }
            if i==j+1 {
                return 1.0
            }
            return 0.0
        });
        let mut x0 = Array1::from_elem(N as usize, 0.0);
        let x_star = Array1::from_shape_fn(N as usize, |i| {
            (i as f64 * h * PI).sin()
        });
        // we want to know how good our solvers are and not the discretisation
        let b = A.dot(&x_star);

        let mut solver = Solver::new(
            vecs,
            PCG::new(
                // scale the Matrix such that ||B||_oo < 1
                NeumannSeries::new(3, &A).scale(0.25*h*h),
                CG::new(N, NoResCeck()),
            )
        );
        solver.solve(&A, &mut x0, &b);

        println!("ERROR {:.3e}", (&x_star - &x0).norm_max());
        assert!((x_star - x0).norm_max() < 1e-13);
    }

    #[test]
    fn solver_bicgstab() {
        let vecs = vec![Array1::from_elem(N as usize, 0.0); 10];

        let mut solver = Solver::new(
            vecs,
            BiCGStab::new(N, Absolute(1e-12))
        );
        test_convergence(&mut solver);
    }

    #[test]
    fn solver_bicgstabm() {
        let vecs = vec![Array1::from_elem(N as usize, 0.0); 10];

        let mut solver = Solver::new(
            vecs,
            Restart::new(
                N,
                BiCGStab::new(12, Absolute(1e-12))
                    .inspect_residual(|k,r| {println!("k: {:>3} r: {:.3e}",k,r)})
            )
        );
        test_convergence(&mut solver);
    }
}

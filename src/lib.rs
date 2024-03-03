#![cfg_attr(
    all(any(target_arch = "x86_64", target_arch = "x86"), feature = "nightly"),
    feature(stdarch_x86_avx512, avx512_target_feature)
)]

use core::mem::MaybeUninit;
use equator::debug_assert;

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[allow(non_camel_case_types)]
pub type c32 = num_complex::Complex32;
#[allow(non_camel_case_types)]
pub type c64 = num_complex::Complex64;

pub struct MicroKernelData<T> {
    pub alpha: T,
    pub beta: T,
    pub conj_lhs: bool,
    pub conj_rhs: bool,
    pub k: usize,
    pub dst_cs: isize,
    pub lhs_cs: isize,
    pub rhs_rs: isize,
    pub rhs_cs: isize,
    pub last_mask: *const (),
}

pub type MicroKernel<T> =
    unsafe fn(data: &MicroKernelData<T>, dst: *mut T, lhs: *const T, rhs: *const T);

#[derive(Debug, Copy, Clone)]
pub struct Plan<T> {
    microkernels: [[MaybeUninit<MicroKernel<T>>; 2]; 2],
    millikernel: unsafe fn(
        microkernels: &[[MaybeUninit<MicroKernel<T>>; 2]; 2],
        mr: usize,
        nr: usize,
        m: usize,
        n: usize,
        k: usize,
        dst: *mut T,
        dst_rs: isize,
        dst_cs: isize,
        lhs: *const T,
        lhs_rs: isize,
        lhs_cs: isize,
        rhs: *const T,
        rhs_rs: isize,
        rhs_cs: isize,
        alpha: T,
        beta: T,
        conj_lhs: bool,
        conj_rhs: bool,
        full_mask: *const (),
        last_mask: *const (),
    ),
    mr: usize,
    nr: usize,
    full_mask: *const (),
    last_mask: *const (),
    m: usize,
    n: usize,
    k: usize,
    dst_cs: isize,
    dst_rs: isize,
    lhs_cs: isize,
    lhs_rs: isize,
    rhs_cs: isize,
    rhs_rs: isize,
}

#[allow(unused_variables)]
unsafe fn noop_millikernel<T: Copy>(
    microkernels: &[[MaybeUninit<MicroKernel<T>>; 2]; 2],
    mr: usize,
    nr: usize,
    m: usize,
    n: usize,
    k: usize,
    dst: *mut T,
    dst_rs: isize,
    dst_cs: isize,
    lhs: *const T,
    lhs_rs: isize,
    lhs_cs: isize,
    rhs: *const T,
    rhs_rs: isize,
    rhs_cs: isize,
    alpha: T,
    beta: T,
    conj_lhs: bool,
    conj_rhs: bool,
    full_mask: *const (),
    last_mask: *const (),
) {
}

#[allow(unused_variables)]
unsafe fn naive_millikernel<
    T: Copy + core::ops::Mul<Output = T> + core::ops::Add<Output = T> + PartialEq,
>(
    microkernels: &[[MaybeUninit<MicroKernel<T>>; 2]; 2],
    mr: usize,
    nr: usize,
    m: usize,
    n: usize,
    k: usize,
    dst: *mut T,
    dst_rs: isize,
    dst_cs: isize,
    lhs: *const T,
    lhs_rs: isize,
    lhs_cs: isize,
    rhs: *const T,
    rhs_rs: isize,
    rhs_cs: isize,
    alpha: T,
    beta: T,
    conj_lhs: bool,
    conj_rhs: bool,
    full_mask: *const (),
    last_mask: *const (),
) {
    let zero: T = core::mem::zeroed();
    if alpha == zero {
        for j in 0..n {
            for i in 0..m {
                let mut acc = zero;
                for depth in 0..k {
                    acc = acc
                        + *lhs.offset(lhs_rs * i as isize + lhs_cs * depth as isize)
                            * *rhs.offset(rhs_rs * depth as isize + rhs_cs * j as isize);
                }
                *dst.offset(dst_rs * i as isize + dst_cs * j as isize) = beta * acc;
            }
        }
    } else {
        for j in 0..n {
            for i in 0..m {
                let mut acc = zero;
                for depth in 0..k {
                    acc = acc
                        + *lhs.offset(lhs_rs * i as isize + lhs_cs * depth as isize)
                            * *rhs.offset(rhs_rs * depth as isize + rhs_cs * j as isize);
                }
                let dst = dst.offset(dst_rs * i as isize + dst_cs * j as isize);
                *dst = alpha * *dst + beta * acc;
            }
        }
    }
}

#[allow(unused_variables)]
unsafe fn fill_millikernel<T: Copy + PartialEq + core::ops::Mul<Output = T>>(
    microkernels: &[[MaybeUninit<MicroKernel<T>>; 2]; 2],
    mr: usize,
    nr: usize,
    m: usize,
    n: usize,
    k: usize,
    dst: *mut T,
    dst_rs: isize,
    dst_cs: isize,
    lhs: *const T,
    lhs_rs: isize,
    lhs_cs: isize,
    rhs: *const T,
    rhs_rs: isize,
    rhs_cs: isize,
    alpha: T,
    beta: T,
    conj_lhs: bool,
    conj_rhs: bool,
    full_mask: *const (),
    last_mask: *const (),
) {
    let zero: T = core::mem::zeroed();
    if alpha == zero {
        for j in 0..n {
            for i in 0..m {
                *dst.offset(dst_rs * i as isize + dst_cs * j as isize) = core::mem::zeroed();
            }
        }
    } else {
        for j in 0..n {
            for i in 0..m {
                let dst = dst.offset(dst_rs * i as isize + dst_cs * j as isize);
                *dst = alpha * *dst;
            }
        }
    }
}

unsafe fn direct_millikernel<T: Copy>(
    microkernels: &[[MaybeUninit<MicroKernel<T>>; 2]; 2],
    mr: usize,
    nr: usize,
    m: usize,
    n: usize,
    k: usize,
    dst: *mut T,
    dst_rs: isize,
    dst_cs: isize,
    lhs: *const T,
    lhs_rs: isize,
    lhs_cs: isize,
    rhs: *const T,
    rhs_rs: isize,
    rhs_cs: isize,
    alpha: T,
    beta: T,
    conj_lhs: bool,
    conj_rhs: bool,
    full_mask: *const (),
    last_mask: *const (),
) {
    debug_assert!(all(lhs_rs == 1, dst_rs == 1));

    let mut data = MicroKernelData {
        alpha,
        beta,
        conj_lhs,
        conj_rhs,
        k,
        dst_cs,
        lhs_cs,
        rhs_rs,
        rhs_cs,
        last_mask,
    };

    let mut i = 0usize;
    while i < m {
        data.last_mask = if i + mr < m { full_mask } else { last_mask };
        let microkernels = microkernels.get_unchecked((i + mr >= m) as usize);
        let dst = dst.offset(i as isize);

        let mut j = 0usize;
        while j < n {
            let microkernel = microkernels
                .get_unchecked((j + nr >= n) as usize)
                .assume_init();

            microkernel(
                &data,
                dst.offset(j as isize * dst_cs),
                lhs.offset(i as isize),
                rhs.offset(j as isize * rhs_cs),
            );

            j += nr;
        }

        i += mr;
    }
}

trait One {
    const ONE: Self;
}

impl One for f32 {
    const ONE: Self = 1.0;
}
impl One for f64 {
    const ONE: Self = 1.0;
}
impl One for c32 {
    const ONE: Self = Self { re: 1.0, im: 0.0 };
}
impl One for c64 {
    const ONE: Self = Self { re: 1.0, im: 0.0 };
}

unsafe fn copy_millikernel<T: Copy + One>(
    microkernels: &[[MaybeUninit<MicroKernel<T>>; 2]; 2],
    mr: usize,
    nr: usize,
    m: usize,
    n: usize,
    k: usize,
    dst: *mut T,
    dst_rs: isize,
    dst_cs: isize,
    lhs: *const T,
    lhs_rs: isize,
    lhs_cs: isize,
    rhs: *const T,
    rhs_rs: isize,
    rhs_cs: isize,
    mut alpha: T,
    beta: T,
    conj_lhs: bool,
    conj_rhs: bool,
    full_mask: *const (),
    last_mask: *const (),
) {
    if dst_rs == 1 && lhs_rs == 1 {
        let gemm_dst = dst;
        let gemm_lhs = lhs;
        let gemm_dst_cs = dst_cs;
        let gemm_lhs_cs = lhs_cs;

        direct_millikernel(
            microkernels,
            mr,
            nr,
            m,
            n,
            k,
            gemm_dst,
            1,
            gemm_dst_cs,
            gemm_lhs,
            1,
            gemm_lhs_cs,
            rhs,
            rhs_rs,
            rhs_cs,
            alpha,
            beta,
            conj_lhs,
            conj_rhs,
            full_mask,
            last_mask,
        );
    } else {
        let mut dst_tmp: MaybeUninit<[T; 32 * 32]> = core::mem::MaybeUninit::uninit();
        let mut lhs_tmp: MaybeUninit<[T; 32 * 32]> = core::mem::MaybeUninit::uninit();

        let dst_tmp = &mut *((&mut dst_tmp) as *mut _ as *mut [[MaybeUninit<T>; 32]; 32]);
        let lhs_tmp = &mut *((&mut lhs_tmp) as *mut _ as *mut [[MaybeUninit<T>; 32]; 32]);

        let gemm_dst_cs = 32;
        let gemm_lhs_cs = 32;

        let mut depth = 0usize;
        while depth < k {
            let depth_bs = Ord::min(32, k - depth);

            let mut i = 0usize;
            while i < m {
                let i_bs = Ord::min(32, m - i);

                let mut j = 0usize;
                while j < n {
                    let j_bs = Ord::min(32, n - j);

                    let gemm_dst = dst_tmp.as_mut_ptr() as *mut T;
                    let gemm_lhs = lhs_tmp.as_ptr() as *mut T;

                    let dst = dst.offset(dst_rs * i as isize + dst_cs * j as isize);
                    let lhs = lhs.offset(dst_rs * i as isize + dst_cs * j as isize);

                    for jj in 0..j_bs {
                        for ii in 0..i_bs {
                            *(gemm_dst.offset(ii as isize + gemm_dst_cs * jj as isize)
                                as *mut MaybeUninit<T>) = *(dst
                                .offset(dst_rs * ii as isize + dst_cs * jj as isize)
                                as *const MaybeUninit<T>);
                        }
                    }
                    for jj in 0..k {
                        for ii in 0..i_bs {
                            *(gemm_lhs.offset(ii as isize + gemm_lhs_cs * jj as isize)
                                as *mut MaybeUninit<T>) = *(lhs
                                .offset(lhs_rs * ii as isize + lhs_cs * jj as isize)
                                as *const MaybeUninit<T>);
                        }
                    }

                    direct_millikernel(
                        microkernels,
                        mr,
                        nr,
                        m,
                        n,
                        k,
                        gemm_dst,
                        1,
                        gemm_dst_cs,
                        gemm_lhs,
                        1,
                        gemm_lhs_cs,
                        rhs,
                        rhs_rs,
                        rhs_cs,
                        alpha,
                        beta,
                        conj_lhs,
                        conj_rhs,
                        full_mask,
                        if i + i_bs == m { last_mask } else { full_mask },
                    );

                    for j in 0..n {
                        for i in 0..m {
                            *(dst.offset(dst_rs * i as isize + dst_cs * j as isize)
                                as *mut MaybeUninit<T>) = dst_tmp[j][i];
                        }
                    }

                    j += j_bs;
                }

                i += i_bs;
            }

            alpha = T::ONE;
            depth += depth_bs;
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod x86 {
    use super::*;
    use equator::debug_assert;

    impl Plan<f32> {
        fn new_f32_scalar(m: usize, n: usize, k: usize, is_col_major: bool) -> Self {
            Self {
                microkernels: [[MaybeUninit::<MicroKernel<f32>>::uninit(); 2]; 2],
                millikernel: naive_millikernel,
                mr: 0,
                nr: 0,
                full_mask: core::ptr::null(),
                last_mask: core::ptr::null(),
                m,
                n,
                k,
                dst_rs: if is_col_major { 1 } else { isize::MIN },
                dst_cs: isize::MAX,
                lhs_rs: if is_col_major { 1 } else { isize::MIN },
                lhs_cs: isize::MAX,
                rhs_cs: isize::MAX,
                rhs_rs: isize::MAX,
            }
        }

        fn new_f32_avx(m: usize, n: usize, k: usize, is_col_major: bool) -> Self {
            let mut microkernels = [[MaybeUninit::<MicroKernel<f32>>::uninit(); 2]; 2];

            let mr = 2 * 8;
            let nr = 4;

            {
                let k = Ord::min(k.wrapping_sub(1), 16);
                let m = (m.wrapping_sub(1) / 8) % (mr / 8);
                let n = n.wrapping_sub(1) % nr;

                microkernels[0][0].write(avx::f32::MICROKERNELS[k][1][3]);
                microkernels[0][1].write(avx::f32::MICROKERNELS[k][1][n]);
                microkernels[1][0].write(avx::f32::MICROKERNELS[k][m][3]);
                microkernels[1][1].write(avx::f32::MICROKERNELS[k][m][n]);
            }

            Self {
                microkernels,
                millikernel: if m == 0 || n == 0 {
                    noop_millikernel
                } else if k == 0 {
                    fill_millikernel
                } else if is_col_major {
                    direct_millikernel
                } else {
                    copy_millikernel
                },
                mr,
                nr,
                m,
                n,
                k,
                dst_rs: if is_col_major { 1 } else { isize::MIN },
                dst_cs: isize::MIN,
                lhs_rs: if is_col_major { 1 } else { isize::MIN },
                lhs_cs: isize::MIN,
                rhs_cs: isize::MIN,
                rhs_rs: isize::MIN,
                full_mask: (&avx::f32::MASKS[0]) as *const _ as *const (),
                last_mask: (&avx::f32::MASKS[m % 8]) as *const _ as *const (),
            }
        }

        #[cfg(feature = "nightly")]
        fn new_f32_avx512(m: usize, n: usize, k: usize, is_col_major: bool) -> Self {
            let mut microkernels = [[MaybeUninit::<MicroKernel<f32>>::uninit(); 2]; 2];

            let mr = 2 * 16;
            let nr = 4;

            {
                let k = Ord::min(k.wrapping_sub(1), 16);
                let m = (m.wrapping_sub(1) / 16) % (mr / 16);
                let n = n.wrapping_sub(1) % nr;

                microkernels[0][0].write(avx512::f32::MICROKERNELS[k][1][3]);
                microkernels[0][1].write(avx512::f32::MICROKERNELS[k][1][n]);
                microkernels[1][0].write(avx512::f32::MICROKERNELS[k][m][3]);
                microkernels[1][1].write(avx512::f32::MICROKERNELS[k][m][n]);
            }

            Self {
                microkernels,
                millikernel: if m == 0 || n == 0 {
                    noop_millikernel
                } else if k == 0 {
                    fill_millikernel
                } else if is_col_major {
                    direct_millikernel
                } else {
                    copy_millikernel
                },
                mr,
                nr,
                m,
                n,
                k,
                dst_rs: if is_col_major { 1 } else { isize::MIN },
                dst_cs: isize::MIN,
                lhs_rs: if is_col_major { 1 } else { isize::MIN },
                lhs_cs: isize::MIN,
                rhs_cs: isize::MIN,
                rhs_rs: isize::MIN,
                full_mask: (&avx512::f32::MASKS[0]) as *const _ as *const (),
                last_mask: (&avx512::f32::MASKS[m % 16]) as *const _ as *const (),
            }
        }

        #[track_caller]
        pub fn new_colmajor_lhs_and_dst_f32(m: usize, n: usize, k: usize) -> Self {
            #[cfg(feature = "nightly")]
            if std::is_x86_feature_detected!("avx512f") {
                return Self::new_f32_avx512(m, n, k, true);
            }

            if std::is_x86_feature_detected!("avx")
                && std::is_x86_feature_detected!("avx2")
                && std::is_x86_feature_detected!("fma")
            {
                return Self::new_f32_avx(m, n, k, true);
            }

            Self::new_f32_scalar(m, n, k, true)
        }

        #[track_caller]
        pub fn new_f32(m: usize, n: usize, k: usize) -> Self {
            #[cfg(feature = "nightly")]
            if std::is_x86_feature_detected!("avx512f") {
                return Self::new_f32_avx512(m, n, k, false);
            }

            if std::is_x86_feature_detected!("avx")
                && std::is_x86_feature_detected!("avx2")
                && std::is_x86_feature_detected!("fma")
            {
                return Self::new_f32_avx(m, n, k, false);
            }

            Self::new_f32_scalar(m, n, k, false)
        }
    }

    impl Plan<f64> {
        fn new_f64_scalar(m: usize, n: usize, k: usize, is_col_major: bool) -> Self {
            Self {
                microkernels: [[MaybeUninit::<MicroKernel<f64>>::uninit(); 2]; 2],
                millikernel: naive_millikernel,
                mr: 0,
                nr: 0,
                full_mask: core::ptr::null(),
                last_mask: core::ptr::null(),
                m,
                n,
                k,
                dst_rs: if is_col_major { 1 } else { isize::MIN },
                dst_cs: isize::MAX,
                lhs_rs: if is_col_major { 1 } else { isize::MIN },
                lhs_cs: isize::MAX,
                rhs_cs: isize::MAX,
                rhs_rs: isize::MAX,
            }
        }

        fn new_f64_avx(m: usize, n: usize, k: usize, is_col_major: bool) -> Self {
            let mut microkernels = [[MaybeUninit::<MicroKernel<f64>>::uninit(); 2]; 2];

            let mr = 2 * 4;
            let nr = 4;

            {
                let k = Ord::min(k.wrapping_sub(1), 16);
                let m = (m.wrapping_sub(1) / 4) % (mr / 4);
                let n = n.wrapping_sub(1) % nr;

                microkernels[0][0].write(avx::f64::MICROKERNELS[k][1][3]);
                microkernels[0][1].write(avx::f64::MICROKERNELS[k][1][n]);
                microkernels[1][0].write(avx::f64::MICROKERNELS[k][m][3]);
                microkernels[1][1].write(avx::f64::MICROKERNELS[k][m][n]);
            }

            Self {
                microkernels,
                millikernel: if m == 0 || n == 0 {
                    noop_millikernel
                } else if k == 0 {
                    fill_millikernel
                } else if is_col_major {
                    direct_millikernel
                } else {
                    copy_millikernel
                },
                mr,
                nr,
                m,
                n,
                k,
                dst_rs: if is_col_major { 1 } else { isize::MIN },
                dst_cs: isize::MIN,
                lhs_rs: if is_col_major { 1 } else { isize::MIN },
                lhs_cs: isize::MIN,
                rhs_cs: isize::MIN,
                rhs_rs: isize::MIN,
                full_mask: (&avx::f64::MASKS[0]) as *const _ as *const (),
                last_mask: (&avx::f64::MASKS[m % 4]) as *const _ as *const (),
            }
        }

        #[cfg(feature = "nightly")]
        fn new_f64_avx512(m: usize, n: usize, k: usize, is_col_major: bool) -> Self {
            let mut microkernels = [[MaybeUninit::<MicroKernel<f64>>::uninit(); 2]; 2];

            let mr = 2 * 8;
            let nr = 4;

            {
                let k = Ord::min(k.wrapping_sub(1), 16);
                let m = (m.wrapping_sub(1) / 8) % (mr / 8);
                let n = n.wrapping_sub(1) % nr;

                microkernels[0][0].write(avx512::f64::MICROKERNELS[k][1][3]);
                microkernels[0][1].write(avx512::f64::MICROKERNELS[k][1][n]);
                microkernels[1][0].write(avx512::f64::MICROKERNELS[k][m][3]);
                microkernels[1][1].write(avx512::f64::MICROKERNELS[k][m][n]);
            }

            Self {
                microkernels,
                millikernel: if m == 0 || n == 0 {
                    noop_millikernel
                } else if k == 0 {
                    fill_millikernel
                } else if is_col_major {
                    direct_millikernel
                } else {
                    copy_millikernel
                },
                mr,
                nr,
                m,
                n,
                k,
                dst_rs: if is_col_major { 1 } else { isize::MIN },
                dst_cs: isize::MIN,
                lhs_rs: if is_col_major { 1 } else { isize::MIN },
                lhs_cs: isize::MIN,
                rhs_cs: isize::MIN,
                rhs_rs: isize::MIN,
                full_mask: (&avx512::f64::MASKS[0]) as *const _ as *const (),
                last_mask: (&avx512::f64::MASKS[m % 8]) as *const _ as *const (),
            }
        }

        #[track_caller]
        pub fn new_colmajor_lhs_and_dst_f64(m: usize, n: usize, k: usize) -> Self {
            #[cfg(feature = "nightly")]
            if std::is_x86_feature_detected!("avx512f") {
                return Self::new_f64_avx512(m, n, k, true);
            }

            if std::is_x86_feature_detected!("avx")
                && std::is_x86_feature_detected!("avx2")
                && std::is_x86_feature_detected!("fma")
            {
                return Self::new_f64_avx(m, n, k, true);
            }

            Self::new_f64_scalar(m, n, k, true)
        }

        #[track_caller]
        pub fn new_f64(m: usize, n: usize, k: usize) -> Self {
            #[cfg(feature = "nightly")]
            if std::is_x86_feature_detected!("avx512f") {
                return Self::new_f64_avx512(m, n, k, false);
            }

            if std::is_x86_feature_detected!("avx")
                && std::is_x86_feature_detected!("avx2")
                && std::is_x86_feature_detected!("fma")
            {
                return Self::new_f64_avx(m, n, k, false);
            }

            Self::new_f64_scalar(m, n, k, false)
        }
    }
    impl Plan<c32> {
        fn new_c32_scalar(m: usize, n: usize, k: usize, is_col_major: bool) -> Self {
            Self {
                microkernels: [[MaybeUninit::<MicroKernel<c32>>::uninit(); 2]; 2],
                millikernel: naive_millikernel,
                mr: 0,
                nr: 0,
                full_mask: core::ptr::null(),
                last_mask: core::ptr::null(),
                m,
                n,
                k,
                dst_rs: if is_col_major { 1 } else { isize::MIN },
                dst_cs: isize::MAX,
                lhs_rs: if is_col_major { 1 } else { isize::MIN },
                lhs_cs: isize::MAX,
                rhs_cs: isize::MAX,
                rhs_rs: isize::MAX,
            }
        }

        fn new_c32_avx(m: usize, n: usize, k: usize, is_col_major: bool) -> Self {
            let mut microkernels = [[MaybeUninit::<MicroKernel<c32>>::uninit(); 2]; 2];

            let mr = 2 * 4;
            let nr = 2;

            {
                let k = Ord::min(k.wrapping_sub(1), 16);
                let m = (m.wrapping_sub(1) / 4) % (mr / 4);
                let n = n.wrapping_sub(1) % nr;

                microkernels[0][0].write(avx::c32::MICROKERNELS[k][1][1]);
                microkernels[0][1].write(avx::c32::MICROKERNELS[k][1][n]);
                microkernels[1][0].write(avx::c32::MICROKERNELS[k][m][1]);
                microkernels[1][1].write(avx::c32::MICROKERNELS[k][m][n]);
            }

            Self {
                microkernels,
                millikernel: if m == 0 || n == 0 {
                    noop_millikernel
                } else if k == 0 {
                    fill_millikernel
                } else if is_col_major {
                    direct_millikernel
                } else {
                    copy_millikernel
                },
                mr,
                nr,
                m,
                n,
                k,
                dst_rs: if is_col_major { 1 } else { isize::MIN },
                dst_cs: isize::MIN,
                lhs_rs: if is_col_major { 1 } else { isize::MIN },
                lhs_cs: isize::MIN,
                rhs_cs: isize::MIN,
                rhs_rs: isize::MIN,
                full_mask: (&avx::c32::MASKS[0]) as *const _ as *const (),
                last_mask: (&avx::c32::MASKS[m % 4]) as *const _ as *const (),
            }
        }

        #[cfg(feature = "nightly")]
        fn new_c32_avx512(m: usize, n: usize, k: usize, is_col_major: bool) -> Self {
            let mut microkernels = [[MaybeUninit::<MicroKernel<c32>>::uninit(); 2]; 2];

            let mr = 2 * 8;
            let nr = 2;

            {
                let k = Ord::min(k.wrapping_sub(1), 16);
                let m = (m.wrapping_sub(1) / 8) % (mr / 8);
                let n = n.wrapping_sub(1) % nr;

                microkernels[0][0].write(avx512::c32::MICROKERNELS[k][1][1]);
                microkernels[0][1].write(avx512::c32::MICROKERNELS[k][1][n]);
                microkernels[1][0].write(avx512::c32::MICROKERNELS[k][m][1]);
                microkernels[1][1].write(avx512::c32::MICROKERNELS[k][m][n]);
            }

            Self {
                microkernels,
                millikernel: if m == 0 || n == 0 {
                    noop_millikernel
                } else if k == 0 {
                    fill_millikernel
                } else if is_col_major {
                    direct_millikernel
                } else {
                    copy_millikernel
                },
                mr,
                nr,
                m,
                n,
                k,
                dst_rs: if is_col_major { 1 } else { isize::MIN },
                dst_cs: isize::MIN,
                lhs_rs: if is_col_major { 1 } else { isize::MIN },
                lhs_cs: isize::MIN,
                rhs_cs: isize::MIN,
                rhs_rs: isize::MIN,
                full_mask: (&avx512::c32::MASKS[0]) as *const _ as *const (),
                last_mask: (&avx512::c32::MASKS[m % 8]) as *const _ as *const (),
            }
        }

        #[track_caller]
        pub fn new_colmajor_lhs_and_dst_c32(m: usize, n: usize, k: usize) -> Self {
            #[cfg(feature = "nightly")]
            if std::is_x86_feature_detected!("avx512f") {
                return Self::new_c32_avx512(m, n, k, true);
            }

            if std::is_x86_feature_detected!("avx")
                && std::is_x86_feature_detected!("avx2")
                && std::is_x86_feature_detected!("fma")
            {
                return Self::new_c32_avx(m, n, k, true);
            }

            Self::new_c32_scalar(m, n, k, true)
        }

        #[track_caller]
        pub fn new_c32(m: usize, n: usize, k: usize) -> Self {
            #[cfg(feature = "nightly")]
            if std::is_x86_feature_detected!("avx512f") {
                return Self::new_c32_avx512(m, n, k, false);
            }

            if std::is_x86_feature_detected!("avx")
                && std::is_x86_feature_detected!("avx2")
                && std::is_x86_feature_detected!("fma")
            {
                return Self::new_c32_avx(m, n, k, false);
            }

            Self::new_c32_scalar(m, n, k, false)
        }
    }
    impl Plan<c64> {
        fn new_c64_scalar(m: usize, n: usize, k: usize, is_col_major: bool) -> Self {
            Self {
                microkernels: [[MaybeUninit::<MicroKernel<c64>>::uninit(); 2]; 2],
                millikernel: naive_millikernel,
                mr: 0,
                nr: 0,
                full_mask: core::ptr::null(),
                last_mask: core::ptr::null(),
                m,
                n,
                k,
                dst_rs: if is_col_major { 1 } else { isize::MIN },
                dst_cs: isize::MAX,
                lhs_rs: if is_col_major { 1 } else { isize::MIN },
                lhs_cs: isize::MAX,
                rhs_cs: isize::MAX,
                rhs_rs: isize::MAX,
            }
        }

        fn new_c64_avx(m: usize, n: usize, k: usize, is_col_major: bool) -> Self {
            let mut microkernels = [[MaybeUninit::<MicroKernel<c64>>::uninit(); 2]; 2];

            let mr = 2 * 2;
            let nr = 2;

            {
                let k = Ord::min(k.wrapping_sub(1), 16);
                let m = (m.wrapping_sub(1) / 2) % (mr / 2);
                let n = n.wrapping_sub(1) % nr;

                microkernels[0][0].write(avx::c64::MICROKERNELS[k][1][1]);
                microkernels[0][1].write(avx::c64::MICROKERNELS[k][1][n]);
                microkernels[1][0].write(avx::c64::MICROKERNELS[k][m][1]);
                microkernels[1][1].write(avx::c64::MICROKERNELS[k][m][n]);
            }

            Self {
                microkernels,
                millikernel: if m == 0 || n == 0 {
                    noop_millikernel
                } else if k == 0 {
                    fill_millikernel
                } else if is_col_major {
                    direct_millikernel
                } else {
                    copy_millikernel
                },
                mr,
                nr,
                m,
                n,
                k,
                dst_rs: if is_col_major { 1 } else { isize::MIN },
                dst_cs: isize::MIN,
                lhs_rs: if is_col_major { 1 } else { isize::MIN },
                lhs_cs: isize::MIN,
                rhs_cs: isize::MIN,
                rhs_rs: isize::MIN,
                full_mask: (&avx::c64::MASKS[0]) as *const _ as *const (),
                last_mask: (&avx::c64::MASKS[m % 2]) as *const _ as *const (),
            }
        }

        #[cfg(feature = "nightly")]
        fn new_c64_avx512(m: usize, n: usize, k: usize, is_col_major: bool) -> Self {
            let mut microkernels = [[MaybeUninit::<MicroKernel<c64>>::uninit(); 2]; 2];

            let mr = 2 * 4;
            let nr = 2;

            {
                let k = Ord::min(k.wrapping_sub(1), 16);
                let m = (m.wrapping_sub(1) / 4) % (mr / 4);
                let n = n.wrapping_sub(1) % nr;

                microkernels[0][0].write(avx512::c64::MICROKERNELS[k][1][1]);
                microkernels[0][1].write(avx512::c64::MICROKERNELS[k][1][n]);
                microkernels[1][0].write(avx512::c64::MICROKERNELS[k][m][1]);
                microkernels[1][1].write(avx512::c64::MICROKERNELS[k][m][n]);
            }

            Self {
                microkernels,
                millikernel: if m == 0 || n == 0 {
                    noop_millikernel
                } else if k == 0 {
                    fill_millikernel
                } else if is_col_major {
                    direct_millikernel
                } else {
                    copy_millikernel
                },
                mr,
                nr,
                m,
                n,
                k,
                dst_rs: if is_col_major { 1 } else { isize::MIN },
                dst_cs: isize::MIN,
                lhs_rs: if is_col_major { 1 } else { isize::MIN },
                lhs_cs: isize::MIN,
                rhs_cs: isize::MIN,
                rhs_rs: isize::MIN,
                full_mask: (&avx512::c64::MASKS[0]) as *const _ as *const (),
                last_mask: (&avx512::c64::MASKS[m % 4]) as *const _ as *const (),
            }
        }

        #[track_caller]
        pub fn new_colmajor_lhs_and_dst_c64(m: usize, n: usize, k: usize) -> Self {
            #[cfg(feature = "nightly")]
            if std::is_x86_feature_detected!("avx512f") {
                return Self::new_c64_avx512(m, n, k, true);
            }

            if std::is_x86_feature_detected!("avx")
                && std::is_x86_feature_detected!("avx2")
                && std::is_x86_feature_detected!("fma")
            {
                return Self::new_c64_avx(m, n, k, true);
            }

            Self::new_c64_scalar(m, n, k, true)
        }

        #[track_caller]
        pub fn new_c64(m: usize, n: usize, k: usize) -> Self {
            #[cfg(feature = "nightly")]
            if std::is_x86_feature_detected!("avx512f") {
                return Self::new_c64_avx512(m, n, k, false);
            }

            if std::is_x86_feature_detected!("avx")
                && std::is_x86_feature_detected!("avx2")
                && std::is_x86_feature_detected!("fma")
            {
                return Self::new_c64_avx(m, n, k, false);
            }

            Self::new_c64_scalar(m, n, k, false)
        }
    }

    impl<T> Plan<T> {
        #[inline(always)]
        pub unsafe fn execute_unchecked(
            &self,
            m: usize,
            n: usize,
            k: usize,
            dst: *mut T,
            dst_rs: isize,
            dst_cs: isize,
            lhs: *const T,
            lhs_rs: isize,
            lhs_cs: isize,
            rhs: *const T,
            rhs_rs: isize,
            rhs_cs: isize,
            alpha: T,
            beta: T,
            conj_lhs: bool,
            conj_rhs: bool,
        ) {
            debug_assert!(m == self.m);
            debug_assert!(n == self.n);
            debug_assert!(k == self.k);
            if self.dst_cs != isize::MIN {
                debug_assert!(dst_cs == self.dst_cs);
            }
            if self.dst_rs != isize::MIN {
                debug_assert!(dst_rs == self.dst_rs);
            }
            if self.lhs_cs != isize::MIN {
                debug_assert!(lhs_cs == self.lhs_cs);
            }
            if self.lhs_rs != isize::MIN {
                debug_assert!(lhs_rs == self.lhs_rs);
            }
            if self.rhs_cs != isize::MIN {
                debug_assert!(rhs_cs == self.rhs_cs);
            }
            if self.rhs_rs != isize::MIN {
                debug_assert!(rhs_rs == self.rhs_rs);
            }

            (self.millikernel)(
                &self.microkernels,
                self.mr,
                self.nr,
                m,
                n,
                k,
                dst,
                dst_rs,
                dst_cs,
                lhs,
                lhs_rs,
                lhs_cs,
                rhs,
                rhs_rs,
                rhs_cs,
                alpha,
                beta,
                conj_lhs,
                conj_rhs,
                self.full_mask,
                self.last_mask,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use equator::assert;

    #[test]
    fn test_kernel() {
        let gen = |_| rand::random::<f32>();
        let a: [[f32; 17]; 3] = core::array::from_fn(|_| core::array::from_fn(gen));
        let b: [[f32; 6]; 4] = core::array::from_fn(|_| core::array::from_fn(gen));
        let c: [[f32; 15]; 4] = core::array::from_fn(|_| core::array::from_fn(gen));
        assert!(std::is_x86_feature_detected!("avx"));
        assert!(std::is_x86_feature_detected!("avx2"));
        assert!(std::is_x86_feature_detected!("fma"));
        let mut dst = c;

        let last_mask: std::arch::x86_64::__m256i = unsafe {
            core::mem::transmute([
                u32::MAX,
                u32::MAX,
                u32::MAX,
                u32::MAX,
                u32::MAX,
                u32::MAX,
                u32::MAX,
                0,
            ])
        };

        let beta = 2.5;
        let alpha = 1.0;

        unsafe {
            avx::f32::matmul_2_4_dyn(
                &MicroKernelData {
                    alpha,
                    beta,
                    conj_lhs: false,
                    conj_rhs: false,
                    k: 3,
                    dst_cs: dst[0].len() as isize,
                    lhs_cs: a[0].len() as isize,
                    rhs_rs: 2,
                    rhs_cs: 6,
                    last_mask: (&last_mask) as *const _ as *const (),
                },
                dst.as_mut_ptr() as *mut f32,
                a.as_ptr() as *const f32,
                b.as_ptr() as *const f32,
            );
        };

        let mut expected_dst = c;
        for i in 0..15 {
            for j in 0..4 {
                let mut acc = 0.0f32;
                for depth in 0..3 {
                    acc = f32::mul_add(a[depth][i], b[j][2 * depth], acc);
                }
                expected_dst[j][i] = f32::mul_add(beta, acc, expected_dst[j][i]);
            }
        }

        assert!(dst == expected_dst);
    }

    #[test]
    fn test_kernel_cplx() {
        let gen = |_| rand::random::<c32>();
        let a: [[c32; 9]; 3] = core::array::from_fn(|_| core::array::from_fn(gen));
        let b: [[c32; 6]; 2] = core::array::from_fn(|_| core::array::from_fn(gen));
        let c: [[c32; 7]; 2] = core::array::from_fn(|_| core::array::from_fn(gen));
        assert!(std::is_x86_feature_detected!("avx"));
        assert!(std::is_x86_feature_detected!("avx2"));
        assert!(std::is_x86_feature_detected!("fma"));

        let last_mask: std::arch::x86_64::__m256i = unsafe {
            core::mem::transmute([
                u32::MAX,
                u32::MAX,
                u32::MAX,
                u32::MAX,
                u32::MAX,
                u32::MAX,
                0,
                0,
            ])
        };

        let beta = c32::new(2.5, 3.5);
        let alpha = c32::new(1.0, 0.0);

        for (conj_lhs, conj_rhs) in [(false, false), (false, true), (true, false), (true, true)] {
            let mut dst = c;
            unsafe {
                avx::c32::matmul_2_2_dyn(
                    &MicroKernelData {
                        alpha,
                        beta,
                        conj_lhs,
                        conj_rhs,
                        k: 3,
                        dst_cs: dst[0].len() as isize,
                        lhs_cs: a[0].len() as isize,
                        rhs_rs: 2,
                        rhs_cs: b[0].len() as isize,
                        last_mask: (&last_mask) as *const _ as *const (),
                    },
                    dst.as_mut_ptr() as *mut c32,
                    a.as_ptr() as *const c32,
                    b.as_ptr() as *const c32,
                );
            };

            let mut expected_dst = c;
            for i in 0..7 {
                for j in 0..2 {
                    let mut acc = c32::new(0.0, 0.0);
                    for depth in 0..3 {
                        let mut a = a[depth][i];
                        let mut b = b[j][2 * depth];
                        if conj_lhs {
                            a = a.conj();
                        }
                        if conj_rhs {
                            b = b.conj();
                        }
                        acc += a * b;
                    }
                    expected_dst[j][i] += beta * acc;
                }
            }

            for (&dst, &expected_dst) in
                core::iter::zip(dst.iter().flatten(), expected_dst.iter().flatten())
            {
                assert!((dst.re - expected_dst.re).abs() < 1e-5);
                assert!((dst.im - expected_dst.im).abs() < 1e-5);
            }
        }
    }

    #[test]
    fn test_kernel_cplx64() {
        let gen = |_| rand::random::<c64>();
        let a: [[c64; 5]; 3] = core::array::from_fn(|_| core::array::from_fn(gen));
        let b: [[c64; 6]; 2] = core::array::from_fn(|_| core::array::from_fn(gen));
        let c: [[c64; 3]; 2] = core::array::from_fn(|_| core::array::from_fn(gen));
        assert!(std::is_x86_feature_detected!("avx"));
        assert!(std::is_x86_feature_detected!("avx2"));
        assert!(std::is_x86_feature_detected!("fma"));

        let last_mask: std::arch::x86_64::__m256i =
            unsafe { core::mem::transmute([u64::MAX, u64::MAX, 0, 0]) };

        let beta = c64::new(2.5, 3.5);
        let alpha = c64::new(1.0, 0.0);

        for (conj_lhs, conj_rhs) in [(false, false), (false, true), (true, false), (true, true)] {
            let mut dst = c;
            unsafe {
                avx::c64::matmul_2_2_dyn(
                    &MicroKernelData {
                        alpha,
                        beta,
                        conj_lhs,
                        conj_rhs,
                        k: 3,
                        dst_cs: dst[0].len() as isize,
                        lhs_cs: a[0].len() as isize,
                        rhs_rs: 2,
                        rhs_cs: b[0].len() as isize,
                        last_mask: (&last_mask) as *const _ as *const (),
                    },
                    dst.as_mut_ptr() as *mut c64,
                    a.as_ptr() as *const c64,
                    b.as_ptr() as *const c64,
                );
            };

            let mut expected_dst = c;
            for i in 0..3 {
                for j in 0..2 {
                    let mut acc = c64::new(0.0, 0.0);
                    for depth in 0..3 {
                        let mut a = a[depth][i];
                        let mut b = b[j][2 * depth];
                        if conj_lhs {
                            a = a.conj();
                        }
                        if conj_rhs {
                            b = b.conj();
                        }
                        acc += a * b;
                    }
                    expected_dst[j][i] += beta * acc;
                }
            }

            for (&dst, &expected_dst) in
                core::iter::zip(dst.iter().flatten(), expected_dst.iter().flatten())
            {
                assert!((dst.re - expected_dst.re).abs() < 1e-5);
                assert!((dst.im - expected_dst.im).abs() < 1e-5);
            }
        }
    }
    #[test]
    fn test_plan() {
        let gen = |_| rand::random::<f32>();
        let m = 31;
        let n = 4;
        let k = 8;

        let a = (0..m * k).into_iter().map(gen).collect::<Vec<_>>();
        let b = (0..k * n).into_iter().map(gen).collect::<Vec<_>>();
        let c = (0..m * n).into_iter().map(|_| 0.0).collect::<Vec<_>>();
        let mut dst = c.clone();

        let plan = Plan::new_colmajor_lhs_and_dst_f32(m, n, k);
        let beta = 2.5;

        unsafe {
            plan.execute_unchecked(
                m,
                n,
                k,
                dst.as_mut_ptr(),
                1,
                m as isize,
                a.as_ptr(),
                1,
                m as isize,
                b.as_ptr(),
                1,
                k as isize,
                1.0,
                beta,
                false,
                false,
            );
        };

        let mut expected_dst = c;
        for i in 0..m {
            for j in 0..n {
                let mut acc = 0.0f32;
                for depth in 0..k {
                    acc = f32::mul_add(a[depth * m + i], b[j * k + depth], acc);
                }
                expected_dst[j * m + i] = f32::mul_add(beta, acc, expected_dst[j * m + i]);
            }
        }

        assert!(dst == expected_dst);
    }

    #[test]
    fn test_plan_cplx() {
        let gen = |_| rand::random::<c64>();
        let m = 4;
        let n = 4;
        let k = 4;

        let a = (0..m * k).into_iter().map(gen).collect::<Vec<_>>();
        let b = (0..k * n).into_iter().map(gen).collect::<Vec<_>>();
        let c = (0..m * n).into_iter().map(gen).collect::<Vec<_>>();

        for alpha in [c64::new(0.0, 0.0), c64::new(1.0, 0.0), c64::new(2.7, 3.7)] {
            let mut dst = c.clone();

            let plan = Plan::new_colmajor_lhs_and_dst_c64(m, n, k);
            let beta = c64::new(2.5, 0.0);

            unsafe {
                plan.execute_unchecked(
                    m,
                    n,
                    k,
                    dst.as_mut_ptr(),
                    1,
                    m as isize,
                    a.as_ptr(),
                    1,
                    m as isize,
                    b.as_ptr(),
                    1,
                    k as isize,
                    alpha,
                    beta,
                    false,
                    false,
                );
            };

            let mut expected_dst = c.clone();
            for i in 0..m {
                for j in 0..n {
                    let mut acc = c64::new(0.0, 0.0);
                    for depth in 0..k {
                        acc += a[depth * m + i] * b[j * k + depth];
                    }
                    expected_dst[j * m + i] = alpha * expected_dst[j * m + i] + beta * acc;
                }
            }

            for (&dst, &expected_dst) in core::iter::zip(dst.iter(), expected_dst.iter()) {
                assert!((dst.re - expected_dst.re).abs() < 1e-5);
                assert!((dst.im - expected_dst.im).abs() < 1e-5);
            }
        }
    }

    #[test]
    fn test_plan_strided() {
        let gen = |_| rand::random::<f32>();
        let m = 31;
        let n = 4;
        let k = 8;

        let a = (0..2 * 33 * k).into_iter().map(gen).collect::<Vec<_>>();
        let b = (0..k * n).into_iter().map(gen).collect::<Vec<_>>();
        let c = (0..3 * 44 * n).into_iter().map(|_| 0.0).collect::<Vec<_>>();
        let mut dst = c.clone();

        let plan = Plan::new_f32(m, n, k);
        let beta = 2.5;

        unsafe {
            plan.execute_unchecked(
                m,
                n,
                k,
                dst.as_mut_ptr(),
                3,
                44,
                a.as_ptr(),
                2,
                33,
                b.as_ptr(),
                1,
                k as isize,
                1.0,
                beta,
                false,
                false,
            );
        };

        let mut expected_dst = c;
        for i in 0..m {
            for j in 0..n {
                let mut acc = 0.0f32;
                for depth in 0..k {
                    acc = f32::mul_add(a[depth * 33 + i * 2], b[j * k + depth], acc);
                }
                expected_dst[j * 44 + i * 3] =
                    f32::mul_add(beta, acc, expected_dst[j * 44 + i * 3]);
            }
        }

        assert!(dst == expected_dst);
    }
}
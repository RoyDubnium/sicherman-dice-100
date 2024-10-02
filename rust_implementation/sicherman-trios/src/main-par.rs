use pyo3::prelude::*;
use itertools::Itertools;
use contest_algorithms::math::fft::convolution;
use std::env;
use std::fs;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicI32, Ordering};
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut sides = 8;
    if args.len() > 1
    {
        sides = match args[args.len()-1].to_string().parse::<i64>()
        {
            Ok(number) => number,
            Err(_) => 8
        };
    }
    sicherman(sides);
}

fn factorise(input : Vec<i64>) -> Vec<Vec<i64>> {
    let mut result : Vec<Vec<i64>> = Vec::new();
    Python::with_gil(|py| {
        let sympy = py.import_bound("sympy").unwrap();
        let x = sympy.call_method1("symbols", ("x",)).unwrap();
        let poly = sympy.call_method1("Poly", (input,&x)).unwrap();
        let polyexp = poly.call_method0("as_expr").unwrap();
        let factors = polyexp.call_method0("factor").unwrap();
        let factors_list = factors.getattr("args").unwrap();
        let factors_len = factors_list.len().unwrap();
        for factor_idx in 0usize..factors_len
        {
            let factor = factors_list.get_item(factor_idx).unwrap();
            let factor_poly = sympy.call_method1("Poly",(factor,)).unwrap();
            let coeffs : Vec<i64> = factor_poly.call_method0("all_coeffs").unwrap().extract().unwrap();
            result.push(coeffs);
        }
    });
    return result;
}
fn repeat_elements(vec: Vec<Vec<i64>>, n: usize) -> Vec<Vec<i64>> {
    vec.into_iter()
        .flat_map(|inner_vec| std::iter::repeat(inner_vec).take(n))
        .collect()
}
fn coeff_to_sides(coeffs : Vec<i64>) -> Vec<i64>
{
    let mut sides = Vec::new();
    for (i, &c) in coeffs.iter().enumerate() {
        if c > 0 {
            sides.extend(vec![i as i64 + 1; c as usize]);
        }
    }
    return sides;
}

fn sicherman(sides: i64) {
    let polyvec = vec![1; sides as usize];
    let polyfactors = factorise(polyvec);
    let polyfactors = repeat_elements(polyfactors,3);
    let factor_length = polyfactors.len();
    println!("{}", factor_length);
    let factor_sums: Vec<i64> = polyfactors.iter().map(|x| x.iter().sum()).collect();
    
    let result_count = Arc::new(AtomicI32::new(0));
    
    let coeffs_list = Arc::new(Mutex::new(Vec::new()));

    for iterlen1 in 0..(1+factor_length/3) {
        let coeffs_list = Arc::clone(&coeffs_list);

        (0..factor_length).combinations(iterlen1)
            .par_bridge() // Convert the iterator to a parallel iterator
            .for_each(|a| {
                let product: i64 = a.iter().map(|&i| factor_sums[i]).product();
                if product != sides {
                    return;
                }
                let b_and_c: Vec<usize> = (0..factor_length).filter(|i| !a.contains(i)).collect();
                for iterlen2 in 1..(1+(factor_length-iterlen1)/2)
                {
                    b_and_c.clone().into_iter().combinations(iterlen2)
                    .par_bridge()
                    .for_each(|b| {
                        let product: i64 = b.iter().map(|&i| factor_sums[i]).product();
                        if product != sides {
                            return;
                        }
                        let c: Vec<usize> = (0..factor_length).filter(|i| !a.contains(i) && !b.contains(i)).collect();
                        let mut ac = polyfactors[a[0]].clone();
                        if a.len() > 1 {
                            for i in a.iter().skip(1) {
                                ac = convolution(&ac, &polyfactors[*i]);
                            }
                        }
                        let mut bc = polyfactors[b[0]].clone();
                        if b.len() > 1 {
                            for i in b.iter().skip(1) {
                                bc = convolution(&bc, &polyfactors[*i]);
                            }
                        }
                        let mut cc = polyfactors[c[0]].clone();
                        if c.len() > 1 {
                            for i in c.iter().skip(1) {
                                cc = convolution(&cc, &polyfactors[*i]);
                            }
                        }
                        
                        if ac.iter().min().unwrap() >= &0 && bc.iter().min().unwrap() >= &0 && cc.iter().min().unwrap() >= &0 && ac.iter().sum::<i64>() == sides && bc.iter().sum::<i64>() == sides && cc.iter().sum::<i64>() == sides {
                            let mut coeffs = vec![ac.clone(), bc.clone(), cc.clone()];
                            coeffs.sort();
                            let coeffs = (coeffs[0].clone(),coeffs[1].clone(),coeffs[2].clone());
                            let mut coeffs_list = coeffs_list.lock().unwrap();
                            if !coeffs_list.iter().any(|i| i == &coeffs) {
                                result_count.store(result_count.load(Ordering::Relaxed)+1, Ordering::Relaxed);
                                println!("{}: {},{},{}", result_count.load(Ordering::Relaxed),a.len(),b.len(),c.len());
                                coeffs_list.push(coeffs);
                            }
                        }
                    });
                }                     
            });
    }

    let mut contents: Vec<String> = Vec::new();
    let coeffs_list = Arc::try_unwrap(coeffs_list).unwrap().into_inner().unwrap();
    let mut results : Vec<(Vec<i64>,Vec<i64>,Vec<i64>)> = Vec::new();
    for (ac,bc,cc) in coeffs_list
    {
        let a4 = coeff_to_sides(ac);
        let b4 = coeff_to_sides(bc);
        let c4 = coeff_to_sides(cc);
        let mut res = vec![a4.clone(), b4.clone(), c4.clone()];
        res.sort();
        let res = (res[0].clone(),res[1].clone(),res[2].clone());
        results.push(res);
    }
    results.sort();
    for res in results {
        let resstr = format!("{:?}", res);
        let mut reschr = resstr.chars();
        reschr.next();
        reschr.next_back();
        contents.push(reschr.as_str().to_string());
    }
    let contents_string = contents.join("\n");
    fs::write(format!("./results/sicherman-d{:03}-test.txt", sides), contents_string).expect("Unable to write file");
}
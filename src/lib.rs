use rand::Rng;
use std::collections::HashMap;

#[macro_export]
macro_rules! warning_continue {
    ($x:expr) => {
        {
        	println!("{}", $x);
            continue
        }
    };
}

pub fn random_pick_list<T>(list: Vec<&T>, weights: Vec<i32>) -> &T {
	// todo 可以使用树状数组更新前缀和，进行多次抽取
	let mut pre_sum = Vec::new();
	let mut sum = 0;
	for w in weights {
		sum += w;
		pre_sum.push(sum);
	}
	if sum <= 0 {
		panic!("抽取失败");
	}
	let mut random = rand::thread_rng();
	let r = random.gen_range(0..sum);
	for i in 0..pre_sum.len() {
		if r < pre_sum[i] {
			return list[i];
		}
	}
	panic!("抽取失败");
}

pub fn random_pick_map<T>(weight_map: &HashMap<T, i32>) -> &T {
	let mut list = Vec::new();
	let mut weights = Vec::new();
	for (key, value) in weight_map {
		list.push(key);
		weights.push(*value);
	}
	random_pick_list(list, weights)
}
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::fmt;
use azul::random_pick_map;
use azul::warning_continue;
use std::io::stdin;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum CeramicTile {
	BLACK,
	RED,
	YELLOW,
	BLUE,
	WHITE
}
impl fmt::Display for CeramicTile {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
        	CeramicTile::BLACK => write!(f, "黑"),
        	CeramicTile::RED => write!(f, "红"),
        	CeramicTile::YELLOW => write!(f, "黄"),
        	CeramicTile::BLUE => write!(f, "蓝"),
        	CeramicTile::WHITE => write!(f, "白")
        }
    }
}

static FACTORY_NO: AtomicUsize = AtomicUsize::new(0);
struct Factory {
	no: usize,
	stock: Option<HashMap<CeramicTile, i32>>
}
impl Factory {
	fn new() -> Factory {
		Factory {
			no: FACTORY_NO.fetch_add(1, Ordering::SeqCst),
			stock: Some(HashMap::new())
		}
	}

	fn sum(&self) -> i32 {
		self.stock.as_ref().unwrap().values().sum::<i32>()
	}

	fn add(&mut self, ceramic_tile: CeramicTile, num: i32) {
		*self.stock.as_mut().unwrap().entry(ceramic_tile).or_insert(0) += num;
	}

	fn catch(&mut self) -> HashMap<CeramicTile, i32> {
		let tmp = self.stock.take();
		self.stock = Some(HashMap::new());
		tmp.unwrap()
	}

	fn contains(&self, ceramic_tile: CeramicTile) -> bool {
		if let Some(&n) = self.stock.as_ref().unwrap().get(&ceramic_tile) {
			if n > 0 {
				return true;
			}
		}
		return false;
	}
}
impl fmt::Display for Factory {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "工厂{}：", self.no)?;
        for (key, value) in self.stock.as_ref().unwrap() {
        	write!(f, "{}={}  ", key, value)?;
    	}
    	writeln!(f)
    }
}


struct Player<'a> {
	name: &'a str, 
	temporary: Vec<(Option<CeramicTile>, i32)>,
	wall: Vec<Vec<(CeramicTile, bool)>>,
	floor: Vec<Option<CeramicTile>>,
	score: i32
}
impl Player<'_> {
	fn new(name: &str) -> Player {
		let wall = vec![
			vec![(CeramicTile::BLUE, false), (CeramicTile::YELLOW, false), (CeramicTile::RED, false), (CeramicTile::BLACK, false), (CeramicTile::WHITE, false)],
			vec![(CeramicTile::WHITE, false), (CeramicTile::BLUE, false), (CeramicTile::YELLOW, false), (CeramicTile::RED, false), (CeramicTile::BLACK, false)],
			vec![(CeramicTile::BLACK, false), (CeramicTile::WHITE, false), (CeramicTile::BLUE, false), (CeramicTile::YELLOW, false), (CeramicTile::RED, false)],
			vec![(CeramicTile::RED, false), (CeramicTile::BLACK, false), (CeramicTile::WHITE, false), (CeramicTile::BLUE, false), (CeramicTile::YELLOW, false)],
			vec![(CeramicTile::YELLOW, false), (CeramicTile::RED, false), (CeramicTile::BLACK, false), (CeramicTile::WHITE, false), (CeramicTile::BLUE, false)],
		];
		let temporary = vec![(None, 0), (None, 0), (None, 0), (None, 0), (None, 0)];
		Player {
			name: name,
			temporary: temporary,
			wall: wall,
			floor: Vec::new(),
			score: 0
		}
	}

	fn can_put(&self, ceramic_tile: CeramicTile, row: Option<usize>) -> bool {
		if let Some(row_n) = row {
			if row_n >= 5 {
				return false;
			}
			if self.temporary[row_n].1 > 0 && self.temporary[row_n].0.unwrap() != ceramic_tile {
				return false;
			}
			// todo 墙上有相同颜色的也不能放
			for x in &self.wall[row_n] {
				if x.0 == ceramic_tile && x.1 {
					return false;
				}
			}
		}
		return true;
	}

	fn set_first(&mut self) {
		self.floor.push(None);
	}

	fn take(&mut self, ceramic_tile: CeramicTile, n: i32, row: Option<usize>) {
		let left;
		if let Some(row_n) = row {
			let max_size = row_n as i32 + 1;
			if n + self.temporary[row_n].1 > max_size {
				left = n + self.temporary[row_n].1 - max_size;
				self.temporary[row_n] = (Some(ceramic_tile), max_size);
			} else {
				left = 0;
				self.temporary[row_n] = (Some(ceramic_tile), n + self.temporary[row_n].1);
			}
		} else {
			left = n;
		}
		for _i in 0..left {
			self.floor.push(Some(ceramic_tile));
		}
	}

	fn liquidate(&mut self) -> (HashMap<CeramicTile, i32>, bool) {
		let one_d_score = |wall: &Vec<Vec<(CeramicTile, bool)>>, i: usize, j: usize, di: i32, dj: i32| {
			let m = wall.len() as i32;
			let n = wall[0].len() as i32;
			let mut i = i as i32;
			let mut j = j as i32;
			let mut res = 0;
			loop {
				i += di;
				j += dj;
				if di < 0 || di >= m || dj < 0 || dj >= n {
					return res;
				}
				if wall[i as usize][j as usize].1 {
					res += 1;
				}
			}
		};
		let mut trash = HashMap::new();
		for i in 0..self.temporary.len() {
			if self.temporary[i].1 == i as i32 + 1 {
				for j in 0..self.wall[i].len() {
					if self.wall[i][j].0 == self.temporary[i].0.unwrap() {
						self.wall[i][j] = (self.wall[i][j].0, true);
						// todo 计分
						break;
					}
				}
				*trash.entry(self.temporary[i].0.unwrap()).or_insert(0) += self.temporary[i].1 - 1;
				self.temporary[i] = (None, 0);
			}
		}
		// 地板排计分
		// todo 计分
		for item in &self.floor {
			if let Some(ceramic_tile) = item.as_ref() {
				*trash.entry(*ceramic_tile).or_insert(0) += 1;
			}
		}
		self.floor = Vec::new();
		// 是否结束
		let is_end = self.wall.iter().any(|row| row.iter().all(|x| x.1));
		return (trash, is_end);
	}
}
impl<'a> fmt::Display for Player<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "玩家{}：{}分", self.name, self.score)?;
        let fmt_tem = |item: &(Option<CeramicTile>, i32)| {
        	let mut s = String::new();
        	if let Some(ceramic_tile) = &item.0 {
        		for _i in 0..(10-2*item.1) {
        			s.push_str(" ");
        		}
        		for _i in 0..item.1 {
        			s.push_str(&format!("{}", ceramic_tile));
        		}
        	} else {
        		s.push_str("          ");
        	}
        	s
        };
        let fmt_wall = |row: &Vec<(CeramicTile, bool)>| {
        	let mut s = String::new();
        	for item in row {
        		if item.1 {
        			s.push_str(&format!("{}", item.0));
        		} else {
        			s.push_str("  ");
        		}
        	}
        	s
        };
        for i in 0..5 {
        	writeln!(f, "{}:{}->{}", i, fmt_tem(&self.temporary[i]), fmt_wall(&self.wall[i]))?;
        }
        writeln!(f, "地板={}", &"-1-1-2-2-2-3-3"[0..2*&self.floor.len().min(8)])
    }
}

struct Game<'a> {
	current_player: usize,
	first_flag: Option<usize>,
	players: Vec<Player<'a>>,
	factories: Vec<Factory>,
	trash: HashMap<CeramicTile, i32>,
	available: HashMap<CeramicTile, i32>,
	public_area: HashMap<CeramicTile, i32>
}
impl<'a> Game<'a> {
	fn new() -> Game<'static> {
		let mut available = HashMap::new();
		let ceramic_tiles_n = 5;
		available.insert(CeramicTile::BLUE, ceramic_tiles_n);
		available.insert(CeramicTile::WHITE, ceramic_tiles_n);
		available.insert(CeramicTile::BLACK, ceramic_tiles_n);
		available.insert(CeramicTile::RED, ceramic_tiles_n);
		available.insert(CeramicTile::YELLOW, ceramic_tiles_n);
		Game {
			current_player: 0,
			first_flag: None,
			players: vec![],
			factories: vec![],
			trash: HashMap::new(),
			available: available,
			public_area: HashMap::new()
		}
	}

	fn regist(&mut self, player: Player<'a>) {
		self.players.push(player);
	}

	fn init(&mut self) {
		// 根据玩家人数初始化工厂
		let n_factory;
		match self.players.len() {
			2 => n_factory = 5,
			3 => n_factory = 7,
			_ => panic!("玩家人数不对")
		}
		for _i in 0..n_factory {
			self.factories.push(Factory::new());
		}
		self.supply();
	}

	fn is_empty(&self) -> bool {
		if self.factories.iter().map(|f| f.sum()).sum::<i32>() > 0 {
			return false;
		}
		if self.public_area.values().sum::<i32>() > 0 {
			return false;
		}
		true
	}

	fn supply(&mut self) {
		if self.available.values().sum::<i32>() < 4*self.factories.len() as i32 {
			for (key, value) in &self.trash {
				*self.available.entry(*key).or_insert(0) += value;
			}
			self.trash.clear();
		}
		for factory in self.factories.iter_mut() {
			for _i in 0..4 {
				let ceramic_tile = *random_pick_map(&self.available);
				factory.add(ceramic_tile, 1);
				*self.available.entry(ceramic_tile).or_insert(0) -= 1;
			}
		}
	}

	fn liquidate(&mut self) -> bool {
		if !self.is_empty() {
			return false;
		}
		// 每个玩家计分
		let mut is_end = false;
		for player in self.players.iter_mut() {
			let (trash, can_end) = player.liquidate();
			if can_end {
				is_end = true;
			}
			// 更新弃牌堆
			for (key, value) in trash {
				*self.trash.entry(key).or_insert(0) += value;
			}
		}
		// 如果游戏还未结束
		if !is_end {
			// 补充货物
			self.supply();
			// 起始玩家为拥有开始标记的玩家
			self.current_player = if let Some(no) = self.first_flag {
				no
			} else {
				0
			}
			self.first_flag = None;
		}
		// todo 处理游戏结束
		return is_end;
	}

	fn step_next(&mut self, factory_no: Option<usize>, ceramic_tile: CeramicTile, row: Option<usize>) -> (bool, bool) {
		let player = &mut self.players[self.current_player];
		if !player.can_put(ceramic_tile, row) {
			return (false, false);
		}
		let n;
		if let Some(no) = factory_no {
			if no >= self.factories.len() {
				return (false, false);
			}
			let factory = &mut self.factories[no];
			if !factory.contains(ceramic_tile) {
				return (false, false);
			}
			let mut ceramic_tiles = factory.catch();
			n = ceramic_tiles.remove(&ceramic_tile).unwrap();
			for (key, value) in ceramic_tiles {
				*self.public_area.entry(key).or_insert(0) += value;
			}
		} else {
			if let Some(&n) = self.public_area.get(&ceramic_tile) {
				if n <= 0 {
					return (false, false);
				}
			} else {
				return (false, false);
			}
			n = self.public_area.remove(&ceramic_tile).unwrap();
			if let None = self.first_flag {
				self.first_flag = Some(self.current_player);
				player.set_first();
			}
		}
		player.take(ceramic_tile, n, row);
		self.current_player = (self.current_player + 1) % self.players.len();
		return (true, self.liquidate());
	}
}
impl<'a> fmt::Display for Game<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "当前玩家{}", self.players[self.current_player].name)?;
        writeln!(f, "1号标记在{}", if let Some(player_no) = self.first_flag { self.players[player_no].name } else { "场中" })?;
        for factory in &self.factories {
        	write!(f, "{}", factory)?;
        }
        write!(f, "场中：")?;
        for (key, value) in &self.public_area {
        	write!(f, "{}={}  ", key, value)?;
    	}
    	writeln!(f)?;
    	write!(f, "牌堆：")?;
        for (key, value) in &self.available {
        	write!(f, "{}={}  ", key, value)?;
    	}
    	writeln!(f)?;
    	write!(f, "弃牌堆：")?;
        for (key, value) in &self.trash {
        	write!(f, "{}={}  ", key, value)?;
    	}
    	writeln!(f)?;
        for player in &self.players {
        	write!(f, "{}", player)?;
        }
        write!(f, "")
    }
}

fn main() {
    let mut game = Game::new();
    game.regist(Player::new("jack"));
    game.regist(Player::new("me"));
    game.init();
    print!("{}", game);
    loop {
    	let mut str_buf = String::new();
	    stdin().read_line(&mut str_buf).expect("Failed to read line.");
	    let inputs = str_buf.trim().split(",").collect::<Vec<&str>>();
	    if inputs.len() != 3 {
	    	warning_continue!("输入的个数需为3");
	    }
	    let factory_no = match inputs[0].trim() {
	    	"" => None,
	    	x => match x.parse::<usize>() {
	    		Ok(num) => Some(num),
            	Err(_) => warning_continue!("工厂编号需要为数字或为空"),
	    	},
	    };
	    let ceramic_tile = match inputs[1].trim() {
	    	"black" => CeramicTile::BLACK,
        	"red" => CeramicTile::RED,
        	"yellow" => CeramicTile::YELLOW,
        	"blue" => CeramicTile::BLUE,
        	"white" => CeramicTile::WHITE,
        	_ => warning_continue!("颜色需为black、red、yellow、blue、white中的一种"),
	    };
	    let row = match inputs[2] {
	    	"" => None,
	    	x => match x.parse::<usize>() {
	    		Ok(num) => Some(num),
            	Err(_) => warning_continue!("行号需要为数字或为空"),
	    	},
	    };
    	let (operate_success, is_end) = game.step_next(factory_no, ceramic_tile, row);
    	if !operate_success {
    		warning_continue!("无法执行的操作");
    	}
    	println!();
    	print!("{}", game);
    }
}

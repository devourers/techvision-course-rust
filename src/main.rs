use itertools::Itertools;
use num::{traits::Pow, clamp};


//static IMAGE_PATH: String = "A".to_string();
static MEAN: f64 = 0.01;
static SIGMA: f64 = 0.005;
static SEED: u64 = 1337;
static MEDIAN_SIZE: usize = 3;
static COLORS: &'static [f32] =
 &[1.0, 1.0, 1.0, 
  1.0, 1.0, 1.0, 
  1.0, 1.0, 1.0];
static ALBEDO: &'static [f32; 9] = 
&[0.5710, 0.6840, 0.8936, 
  0.7646, 0.8089, 0.6404, 
  1.0000, 0.6245, 0.7684];
static SIZE: usize = 300;
static LIGHT_LUMINOSITY: f32 = 1.0;
static DIAG: f32 = (SIZE * 9 * SIZE + SIZE* 9 * SIZE) as f32;
static ITER_TAKE: usize = 5000;
const NTHREADS: usize = 12;
static LOCATIONS: &'static [(usize, usize)] = 
&[(0, 0), 
  (0, 1), 
  (0, 2), 
  (1, 0), 
  (1, 1), 
  (1, 2), 
  (2, 0), 
  (2, 1),
  (2, 2)];
static DIRECTIONS: &'static [(i32, i32)] = &[
 (-1, 1), 
 (1, 1),
 (1, 0), 
 (0, 1), 
 (-1, 0), 
 (0, -1), 
 (-1, -1)
];

fn main(){
    if std::env::args().len() == 1{
        let sim_app = LightSimApp::init(SIZE, ALBEDO);
        let options = eframe::NativeOptions::default();
        eframe::run_native("LightSim", options, Box::new(|_cc| Box::new(sim_app)));
    }
    else{
        let (x_, y_, h_, albedo) = parse_args(&std::env::args().nth(1).unwrap());
        reverse_solve_nomad(x_, y_, h_, &albedo);
    }
}

fn parse_args(arg: &str) -> (i32, i32, u32, [f32;9]){
    //args are x, y, height, albedo1, albedo2, albedo3, albedo4, abledo5, albedo6, albedo7, abledo8, albedo9
    let contents = std::fs::read_to_string(arg).unwrap() as String;
    let contents = contents.trim();
    let splitted_contetns: Vec<&str> = contents.split(' ').collect();
    let x_ = splitted_contetns[0].parse::<i32>().unwrap();
    let y_ = splitted_contetns[1].parse::<i32>().unwrap();
    let h_ = splitted_contetns[2].parse::<u32>().unwrap();
    let a1_ = splitted_contetns[3].parse::<f32>().unwrap();
    let a2_ = splitted_contetns[4].parse::<f32>().unwrap();
    let a3_ = splitted_contetns[5].parse::<f32>().unwrap();
    let a4_ = splitted_contetns[6].parse::<f32>().unwrap();
    let a5_ = splitted_contetns[7].parse::<f32>().unwrap();
    let a6_ = splitted_contetns[8].parse::<f32>().unwrap();
    let a7_ = splitted_contetns[9].parse::<f32>().unwrap();
    let a8_ = splitted_contetns[10].parse::<f32>().unwrap();
    let a9_ = splitted_contetns[11].parse::<f32>().unwrap();
    let albedo: [f32; 9] = [a1_, a2_, a3_, 
                            a4_, a5_, a6_,
                            a7_, a8_, a9_];
    return (x_, y_, h_, albedo);

}

fn reverse_solve_nomad(x_: i32, y_: i32, h_: u32, albedo: &[f32; 9])
{
    let mut lightsimapp = LightSimApp::init(SIZE, &albedo);
    lightsimapp.light_source.coordinates.0 = x_;
    lightsimapp.light_source.coordinates.1 = y_;
    lightsimapp.light_source.height = h_;
    lightsimapp.light_source.is_on = true;
    lightsimapp.update_no_reverse_solve();
    //load 2 images and count distance
    let mut diff = 0.0;
    let img_generated = image::open("scene.png").unwrap().grayscale();
    let img_gen = img_generated.as_luma8().unwrap();
    let img_original = image::open("mondrian_albedo_estimation_frame_3.png").unwrap().grayscale();
    let img_orig = img_original.as_luma8().unwrap();
    for i in 0..SIZE*3{
        for j in 0..SIZE*3{
            diff += (srgb_to_clinear(img_gen.get_pixel(i as u32, j as u32).0[0] as usize) - srgb_to_clinear(img_orig.get_pixel(i as u32, j as u32).0[0] as usize)).pow(2) as f64;
        }
    }
    println!("{}", diff);
}

fn eucl_dist(a: &(i32, i32), b: &(i32, i32)) -> f32{
    let a_ = (a.0 as f32, a.1 as f32);
    let b_ = (b.0 as f32, b.1 as f32);
    let mut dist = (b_.0 - a_.0) * (b_.0 - a_.0) + (b_.1 - a_.1) * (b_.1 - a_.1);
    dist = dist.sqrt();
    return dist;
}

fn get_circle_center(x1: &(i32, i32), x2: &(i32, i32), x3: &(i32, i32)) -> (bool, (i32, i32)){
    //x0
    //count y_bracket once, multiply in up 
    let y1bracket = (x2.0 * x2.0 + x2.1 * x2.1) as i32 - (x3.0 * x3.0) as i32 - (x3.1 * x3.1) as i32;
    let y2bracket = (x3.0 * x3.0 + x3.1 * x3.1) as i32 - (x1.0 * x1.0) as i32 - (x1.1 * x1.1) as i32;
    let y3bracket = (x1.0 * x1.0 + x1.1 * x1.1) as i32 - (x2.0 * x2.0) as i32 - (x2.1 * x2.1) as i32;
    let up = (x1.1 as i32) * y1bracket + (x2.1 as i32) * y2bracket + (x3.1 as i32) * y3bracket;
    let down = x1.0 as i32 * (x2.1 as i32 - x3.1 as i32) + x2.0 as i32*(x3.1 as i32 - x1.1 as i32) + x3.0 as i32*(x1.1 as i32 - x2.1 as i32);
    let mut x = (up as f32) / (down as f32);
    x *= -0.5;
    let up = (x1.0 as i32) *  y1bracket + (x2.0 as i32) * y2bracket + (x3.0 as i32) * y3bracket;
    let mut y = (up as f32) / (down as f32);
    y *= 0.5;
    let mut approved = true;
    if down == 0{
        approved = false;
    }
    //if x <= -0.5 || y <= -0.5 || x >= ((SIZE * 3) as f32 - 0.5) || y >= ((SIZE * 3) as f32 - 0.5) {
    //    approved = false;
    //}
    let x_ = x.round() as i32;
    let y_ = y.round() as i32;
    return (approved, (x_, y_));
}

fn process_patch(cluster: std::collections::HashMap<usize, Vec<(i32, i32)>>) -> std::collections::HashMap<(i32, i32), usize>{
    let mut children = vec!();
    let mut answers: std::collections::HashMap<(i32, i32), usize> = std::collections::HashMap::new();
    //mapreduce
    for cl in cluster{
        children.push(std::thread::spawn(move || -> std::collections::HashMap<(i32, i32), usize> {
            let mut cur_answers: std::collections::HashMap<(i32, i32), usize> = std::collections::HashMap::new();
            let points = cl.1;
            let it = points.iter().combinations(3).take(ITER_TAKE);
            it.for_each(|i|{
                let cur_ans = get_circle_center(i[0], i[1], i[2]);
                if cur_ans.0 == true{
                    if cur_answers.contains_key(&cur_ans.1){
                        *cur_answers.get_mut(&cur_ans.1).unwrap() += 1;
                    }
                    else{
                        cur_answers.insert(cur_ans.1, 1);
                    }
                }
            });
            return cur_answers;
        }));
    }
    let final_result = 
    children.into_iter().map(|c| c.join().unwrap());
    for a in final_result{
        for k in a.keys(){
            if answers.contains_key(k){
                *answers.get_mut(k).unwrap() += 1;
            }
            else{
                answers.insert(*k, *a.get(k).unwrap());
            }
        }
    }
    return answers;
}


fn filter_single_value(patch: &ndarray::Array2::<f32>) -> f32{
    let mut arr: Vec<f32> = vec!();
    for i in 0..MEDIAN_SIZE{
        for j in 0..MEDIAN_SIZE{
            arr.push(patch[[i, j]]);
        }
    }
    arr.sort_by(|a, b| a.partial_cmp(b).unwrap());

    return arr[(arr.len() + 1) /2];
}

fn median_filter_image(array: &ndarray::Array2::<f32>) -> ndarray::Array2::<f32>{
    let mut arr = ndarray::Array2::<f32>::default((array.shape()[0], array.shape()[1]));
    ndarray::Zip::indexed(arr.outer_iter_mut()).par_for_each(|j, mut row| {
        for (k, col) in row.iter_mut().enumerate(){
            let mut curr_patch = ndarray::Array2::<f32>::default((MEDIAN_SIZE, MEDIAN_SIZE));
            for l in 0..MEDIAN_SIZE{
                let mut l_i = (j  as i32) - ((MEDIAN_SIZE+1) / 2 + l) as i32;
                if l_i < 0{
                    l_i = 0;
                }
                if l_i > (SIZE * 3 - 1) as i32{
                    l_i = (SIZE * 3  - 1) as i32;
                }
                for m in 0..MEDIAN_SIZE{
                    let mut m_j = (k as i32) - ((MEDIAN_SIZE+1) / 2 + l) as i32;
                    if m_j < 0{
                        m_j = 0;
                    }
                    if m_j > (SIZE * 3 - 1) as i32{
                        m_j = (SIZE * 3 - 1) as i32;
                    }
                    curr_patch[[l, m]] = array[[l_i as usize, m_j as usize]];
                }
            }
            *col = filter_single_value(&curr_patch);
        }
    });
    return arr;
}


fn launch_ray(reverse_solution_location: &(i32, i32), direction: &(i32, i32), scene_arr: &ndarray::Array2::<f32>) -> (f32, f32){
    //fix me
    let mut res = (0.0, 0.0); //height, diff
        let mut curr_pos = (reverse_solution_location.0, reverse_solution_location.1);
        let mut mov_pos = curr_pos;
        let pic_center = ((SIZE + SIZE / 2) as i32, (SIZE + SIZE / 2) as i32);
        let check_dist = eucl_dist(&pic_center, &mov_pos);
        mov_pos = (mov_pos.0 + direction.0, mov_pos.1 + direction.1);
        let check_dist_2 = eucl_dist(&pic_center, &mov_pos);
        let mut clicks = 0;
        if check_dist_2 < check_dist{
            while !within_bound(mov_pos) && clicks < 1000{
                mov_pos = (mov_pos.0 + direction.0, mov_pos.1 + direction.1);
                clicks +=1;
            }
            while within_bound(mov_pos){
                let new_pos = (mov_pos.0 + direction.0, mov_pos.1 + direction.1);
                if within_bound(new_pos){
                    if get_patch(new_pos.0 as usize, new_pos.1 as usize) == 
                    get_patch(curr_pos.0 as usize, curr_pos.1 as usize){
                        let cur_h = solve_eq(reverse_solution_location, (curr_pos.0 as usize, curr_pos.1 as usize),
                                                        (new_pos.0 as usize, new_pos.1 as usize), 
                                                        scene_arr);
                        if cur_h.1 > res.1{
                            res = cur_h;
                        }
                    }
                    else{
                        curr_pos = new_pos;
                    }
                    mov_pos = (mov_pos.0 + direction.0, mov_pos.1 + direction.1);
                }
                else{
                    break;
                }
            }        
        }
        return res;
}

struct LightSimApp{
    light_source: LightSource,
    scene: Scene,
    noise: Noise,
    img_gui: egui_extras::RetainedImage,
    reverse_solution_height: u32,
    revere_solution_albedo: Vec<f32>,
    reverse_solution_location: (i32, i32),
    scene_arr: ndarray::Array2::<f32>
}


fn load_im_egui() -> eframe::epaint::ColorImage{
    let img = image::open("scene.png").unwrap().to_rgba8();
    let rgba = img.as_raw();
    let img_ = eframe::epaint::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], rgba);
    return img_;
}


fn solve_eq(ls: &(i32, i32), center: (usize, usize), edge: (usize, usize), scene_arr: &ndarray::Array2::<f32>) -> (f32, f32){
    let b2 = scene_arr[[center.0, center.1]].powf(1.0 / 3.0);
    let b1 = scene_arr[[edge.0, edge.1]].powf(1.0 / 3.0);
    let diff = b2 - b1;
    let r1 = eucl_dist(&ls, &(edge.0 as i32, edge.1 as i32));
    let r2 = eucl_dist(&ls, &(center.0 as i32, center.1 as i32));
    let up = b2 * b2 * r2 * r2 - b1 * b1 * r1 * r1;
    let down = (b1 * b1 - b2 * b2) as f32;
    let mut h = up.abs() / down.abs();
    h = h.sqrt();
    return (h, diff.abs());
}


//checks if absolute coordinates are within bounds
fn within_bound(loc: (i32, i32)) -> bool{
    if loc.0 < 0 || loc.1 < 0 || loc.0 > (SIZE*3 - 1) as i32 || loc.1 > (SIZE*3 - 1) as i32{
        return false;
    }
    else{
        return true;
    }
}


//gets patch relative to coordinates
fn get_patch(j: usize, k: usize) -> (usize, usize){
    let mut x = k / SIZE;
    let mut y = j / SIZE;
    if x > 2{
        x = 2;
    }
    if y > 2{
        y = 2;
    }
    return (x, y);
}


//Light simulation app implementation
impl LightSimApp{
    fn init(sz: usize, alb: &[f32; 9]) -> Self{
        let ls = LightSource::init(alb);
        let sc = Scene::init(sz);
        let img_ = load_im_egui();
        let rev_sol_h = 0;
        let rev_sol_loc = (0, 0);
        let rev_sol_albed: Vec<f32> = [0.0, 0.0, 0.0, 
                                       0.0, 0.0, 0.0, 
                                       0.0, 0.0, 0.0].to_vec();
        let shape = (sz*3, sz*3);
        let arr = ndarray::Array2::<f32>::default(shape);
        let ns = Noise::init();
        return LightSimApp { 
            light_source: ls, 
            scene: sc,
            noise: ns,
            reverse_solution_location: rev_sol_loc,
            img_gui: egui_extras::RetainedImage::from_color_image("sceneimg", img_),
            reverse_solution_height: rev_sol_h,
            revere_solution_albedo: rev_sol_albed,
            scene_arr: arr
        }
    }


    //get color clusters from patch
    fn clusterize_patch(&mut self, loc: &(usize, usize)) -> (bool, std::collections::HashMap<usize, Vec<(i32, i32)>>){
        let loc_restrictions = (loc.1 * SIZE, loc.0*SIZE);
        let patch = 
        self.scene_arr.slice(ndarray::s![loc_restrictions.0..loc_restrictions.0+SIZE, 
                                            loc_restrictions.1..loc_restrictions.1+SIZE]);
        let mut clusters: std::collections::HashMap<usize, Vec<(i32, i32)>> = std::collections::HashMap::new();
        let shape = patch.shape();
        //mapreduce?
        //let (min, max) = find_min_max(&patch.to_owned());
        let mut eligible = true;
        //if (max - min).abs() < 0.01{
        //    eligible = false;
        //}
        if eligible{
            for i in 0..shape[0]{
                for j in 0..shape[1]{
                    let current_brightness = (((patch[[i, j]] * 10000.0).round() / 10000.0) * 100000000.0) as usize;
                    if clusters.contains_key(&current_brightness){
                        clusters.get_mut(&current_brightness).unwrap().push(((loc.1 * SIZE + i) as i32, (loc.0 * SIZE + j) as i32));
                    }
                    else if  clusters.keys().len() < 5 * NTHREADS{
                        clusters.insert(current_brightness, Vec::new());
                    }
                }
            }
        }   
        return (eligible, clusters);
    }

    fn update_(&mut self){
        self.light_source.generate_light_matrix();
        self.scene_arr = self.scene.update(&self.light_source, &self.noise);
        if self.noise.is_on{
            self.scene_arr = median_filter_image(&self.scene_arr);
        }
        self.solve_loc();
        self.solve_height();
        self.solve_albedo();
        let img_ = load_im_egui();
        self.img_gui = egui_extras::RetainedImage::from_color_image("sceneimg", img_);
    }


    fn update_no_pic(&mut self){
        self.solve_loc();
        self.solve_height();
        self.solve_albedo();
    }


    fn update_no_reverse_solve(&mut self){
        self.light_source.generate_light_matrix();
        self.scene_arr = self.scene.update(&self.light_source, &self.noise);
        if self.noise.is_on{
            self.scene_arr = median_filter_image(&self.scene_arr);
        }
        let img_ = load_im_egui();
        self.img_gui = egui_extras::RetainedImage::from_color_image("sceneimg", img_);
    }

    fn solve_loc(&mut self){
        let mut answers: std::collections::HashMap<(i32, i32), usize> = std::collections::HashMap::new();
        for loc in LOCATIONS{
            let (valid, clusters) = self.clusterize_patch(loc);
            if valid{
                let pts = process_patch(clusters);
                for pt in pts{
                    if answers.contains_key(&pt.0){
                        *answers.get_mut(&pt.0).unwrap() += pt.1;
                    }
                    else{
                        answers.insert(pt.0, pt.1);
                    }
                }
            }
        }
        let max_elem = answers.iter().max_by_key(|entry| entry.1);
        if !max_elem.is_none(){
            self.reverse_solution_location = *max_elem.unwrap().0;
        }
    }


    fn count_diff_albedo(&mut self, pointf1: (usize, usize), pointf2: (usize, usize)) -> f32{
        let mut f1_b = self.scene_arr[[pointf1.0, pointf1.1]];
        let r1 = eucl_dist(&self.reverse_solution_location, &(pointf1.0 as i32, pointf1.1 as i32));
        let cos1 = self.reverse_solution_height as f32 / (r1*r1 + self.reverse_solution_height as f32*self.reverse_solution_height as f32).sqrt();
        let mut f2_b = self.scene_arr[[pointf2.0, pointf2.1]];
        let r2 = eucl_dist(&self.reverse_solution_location, &(pointf2.0 as i32, pointf2.1 as i32));
        let cos2 = self.reverse_solution_height as f32 / (r2*r2 + self.reverse_solution_height as f32*self.reverse_solution_height as f32).sqrt();
        f1_b /= cos1.pow(3);
        f2_b /= cos2.pow(3);
        return f1_b / f2_b;
    }

    fn solve_height(&mut self){
        let mut h_vec: Vec<(f32, f32)> = vec!();
        let mut children = vec!();
        let loc_copy = self.reverse_solution_location.clone();
        for dir in DIRECTIONS{
            let arr_copy = self.scene_arr.clone();
            children.push(std::thread::spawn(move || -> (f32, f32){
                let h_c = launch_ray(&loc_copy, dir, &arr_copy);
                return h_c;
            }));
        }
        let hs = children.into_iter().map(|c| c.join().unwrap());
        for h in hs{
            if h.1 != 0.0{
                h_vec.push(h);
            }
        }
        h_vec.sort_by(|a, b| a.1.total_cmp(&b.1));
        if h_vec.len() > 0{
            self.reverse_solution_height = h_vec[h_vec.len() - 1].0.round() as u32;
        }
    }


    fn solve_albedo(&mut self){
        let mut albedo_arr = ndarray::Array2::<f32>::default([9, 9]);

        for i in 0..9{
            albedo_arr[[i, i]] = 1.0;
        }
        
        //this is faster since it works in O(1), no need for ifs and copying of array
        let f_12 = self.count_diff_albedo((SIZE-1, SIZE/2), (SIZE, SIZE/2));
        albedo_arr[[0, 1]] = f_12;
        albedo_arr[[1, 0]] = 1.0 / f_12;

        let f_14 = self.count_diff_albedo((SIZE/2, SIZE - 1), (SIZE/2, SIZE));
        albedo_arr[[0, 3]]  = f_14;
        albedo_arr[[3, 0]] = 1.0 / f_14;

        let f_15 = self.count_diff_albedo((SIZE-1, SIZE-1), (SIZE, SIZE));
        albedo_arr[[0, 4]] = f_15;
        albedo_arr[[4, 0]] = 1.0 / f_15;

        let f_23 = self.count_diff_albedo((SIZE*2 - 1, SIZE/2), (SIZE*2, SIZE/2));
        albedo_arr[[1, 2]] = f_23;
        albedo_arr[[2, 1]] = 1.0 / f_23;

        let f_24 = self.count_diff_albedo((SIZE, SIZE-1), (SIZE - 1, SIZE));
        albedo_arr[[1, 3]] = f_24;
        albedo_arr[[3, 1]] = 1.0 / f_24;

        let f_25 = self.count_diff_albedo((SIZE + SIZE/2, SIZE-1), (SIZE + SIZE/2, SIZE));
        albedo_arr[[1, 4]] = f_25;
        albedo_arr[[4, 1]] = 1.0 / f_25;

        let f_26 = self.count_diff_albedo((SIZE * 2 - 1, SIZE-1), (SIZE * 2, SIZE));
        albedo_arr[[1, 5]] = f_26;
        albedo_arr[[5, 1]] = 1.0 / f_26;

        let f_35 = self.count_diff_albedo((SIZE*2, SIZE-1), (SIZE*2-1, SIZE));
        albedo_arr[[2, 4]] = f_35;
        albedo_arr[[4, 2]] = 1.0 / f_35;

        let f_36 = self.count_diff_albedo((SIZE*2 + SIZE / 2, SIZE-1), (SIZE*2 + SIZE/2, SIZE));
        albedo_arr[[2, 5]] = f_36;
        albedo_arr[[5, 2]] = 1.0 / f_36;

        let f_45 = self.count_diff_albedo((SIZE-1, SIZE + SIZE / 2), (SIZE, SIZE + SIZE / 2));
        albedo_arr[[3, 4]] = f_45;
        albedo_arr[[4, 3]] = 1.0 / f_45;

        let f_48 = self.count_diff_albedo((SIZE - 1, 2*SIZE - 1), (SIZE, 2*SIZE));
        albedo_arr[[3, 7]] = f_48;
        albedo_arr[[7, 3]] = 1.0 / f_48;

        let f_47 = self.count_diff_albedo((SIZE/2, SIZE*2 - 1), (SIZE/2, SIZE*2));
        albedo_arr[[3, 6]] = f_47;
        albedo_arr[[6, 3]] = 1.0 / f_47;

        let f_56 = self.count_diff_albedo((SIZE*2 - 1, SIZE + SIZE / 2), (SIZE*2, SIZE + SIZE / 2));
        albedo_arr[[4, 5]] = f_56;
        albedo_arr[[5, 4]] = 1.0 / f_56;

        let f_57 = self.count_diff_albedo((SIZE, SIZE*2 - 1), (SIZE-1, SIZE*2));
        albedo_arr[[4, 6]] = f_57;
        albedo_arr[[6, 4]] = 1.0 / f_57;

        let f_58 = self.count_diff_albedo((SIZE + SIZE/2, SIZE*2 - 1), (SIZE + SIZE/2, SIZE*2));
        albedo_arr[[4, 7]] = f_58;
        albedo_arr[[7, 4]] = 1.0 / f_58;

        let f_59 = self.count_diff_albedo((SIZE * 2 - 1, SIZE*2 - 1), (SIZE * 2, SIZE * 2));
        albedo_arr[[4, 8]] = f_59;
        albedo_arr[[8, 4]] = 1.0 / f_59;

        let f_68 = self.count_diff_albedo((SIZE * 2, SIZE*2 - 1), (SIZE * 2 - 1, SIZE * 2));
        albedo_arr[[5, 7]] = f_68;
        albedo_arr[[7, 5]] = 1.0 / f_68;

        let f_69 = self.count_diff_albedo((SIZE * 2 + SIZE / 2, SIZE*2 - 1), (SIZE * 2 + SIZE / 2, SIZE * 2));
        albedo_arr[[5, 8]] = f_69;
        albedo_arr[[8, 5]] = 1.0 / f_69;

        let f_78 = self.count_diff_albedo((SIZE - 1, SIZE * 2 + SIZE / 2), (SIZE, SIZE * 2 + SIZE / 2));
        albedo_arr[[6, 7]] = f_78;
        albedo_arr[[7, 6]] = 1.0 / f_78;

        let f_89 = self.count_diff_albedo((SIZE * 2 - 1, SIZE * 2 + SIZE / 2), (SIZE * 2, SIZE * 2 + SIZE / 2));
        albedo_arr[[7, 8]] = f_89;
        albedo_arr[[8, 7]] = 1.0 / f_89;


        for i in 0..9{
            for j in 0..9{
                if albedo_arr[[i, j]] == 0.0{
                    albedo_arr[[i, j]] = albedo_arr[[i, 4]] * albedo_arr[[4, j]];
                }
            }
        }

        //full array
        let mut max: usize = 0;
        let mut cur_greater = 0;
        let mut min: usize = 0;
        let mut cur_smaller = 0;
        for (i, r) in albedo_arr.outer_iter().enumerate(){
            let mut small_count = 0;
            let mut large_count = 0;
            for (j, c) in r.iter().enumerate(){
                if i != j{
                    if c < &1.0{
                        small_count += 1;
                    }
                    else {
                        large_count += 1;
                    }
                }
            }
            if large_count > cur_greater{
                max = i;
                cur_greater = large_count;
            } 
            if small_count > cur_smaller{
                min = i;
                cur_smaller = small_count;
            }
        }
        //given max albedo can't be higher than 1.0
        self.revere_solution_albedo[min] = albedo_arr[[min, max]];
        for i in 0..9{
            //self.revere_solution_albedo[i] = albedo_arr[[min, max]] * albedo_arr[[i, min]];
            self.revere_solution_albedo[i] = albedo_arr[[i, max]];
        }
        //minimum is counted via
    }
    
}

struct LightSource{
    location: (usize, usize),
    coordinates: (i32, i32),
    height: u32, //in pixels
    is_on: bool,
    albedo: [f32; 9],
    size: usize,
    light_matrix: ndarray::Array2::<f32>
}


fn get_actual_location(coordinates: (i32, i32), location: (usize, usize), size: usize) -> (i32, i32){
    return ((location.1 * size) as i32 + coordinates.0, (location.0 * size) as i32 + coordinates.1);
}


fn get_light(coordinates: (i32, i32), location: (usize, usize), height: u32, size: usize, j: usize, k: usize, albedo: &[f32]) -> f32{
    let actual_location = get_actual_location(coordinates, location, size);
    if height == 0{
        return 0.0;
    }
    let ground_dist = eucl_dist(&actual_location, &(j as i32, k as i32));
    let tg_a = ground_dist / (height as f32);
    let alpha = tg_a.atan();
    let mut x = k / size;
    let mut y = j / size;
    if x > 2{
        x = 2;
    }
    if y > 2{
        y = 2;
    }
    return alpha.cos().pow(3) * LIGHT_LUMINOSITY * albedo[3 * x + y];
}

impl LightSource{
    fn init(albedo_: &[f32; 9]) -> Self{
        let location_ = (0, 0);
        let height_: u32 = 0;
        let shape = (SIZE*3, SIZE*3);
        let light_matrix_ = ndarray::Array2::<f32>::default(shape);
        let is_on_ = false;
        return LightSource { 
            location: location_,
            coordinates: (0, 0),
            height: height_, 
            size: SIZE,
            albedo: *albedo_,
            light_matrix: light_matrix_,
            is_on: is_on_ 
        };
    }

    fn generate_light_matrix(&mut self){
        if self.is_on{
            ndarray::Zip::indexed(self.light_matrix.outer_iter_mut()).par_for_each(|j, mut row| {
                for (k, col) in row.iter_mut().enumerate(){
                    *col = get_light(self.coordinates, self.location, self.height, self.size, j, k, &self.albedo);
                }
            });
        }
    }

}


struct Noise{
    noise_array: ndarray::Array2::<f32>,
    mean: f64,
    sigma: f64,
    seed: u64,  
    gen: probability::distribution::Gaussian,
    is_on: bool
}

impl Noise{
    fn init() -> Noise{
        let mut source = probability::source::default(SEED);
        let distr = probability::distribution::Gaussian::new(MEAN, SIGMA);
        let sampler = probability::sampler::Independent(&distr, &mut source);
        let values = sampler.take(SIZE*SIZE*9).collect::<Vec<_>>();
        let mut n_a = ndarray::Array2::<f32>::default((SIZE * 3, SIZE*3));
        for i in 0..n_a.shape()[0]{
            for j in 0..n_a.shape()[1]{
                n_a[[i, j]] = values[SIZE*3*j + i] as f32;
            }
        }
        return Noise { 
            noise_array: n_a, 
            mean: MEAN, 
            sigma: SIGMA, 
            seed: SEED, 
            gen: distr, 
            is_on: false 
        }
    }
}


struct Scene{
    scene_array: ndarray::Array2::<f32>,
    scene_image: image::GrayImage,
    size: usize
}

fn decide(sz: usize, j: usize, k:usize) -> f32{
    let mut x = k / sz;
    let mut y = j / sz;
    if x > 2{
        x = 2;
    }
    if y > 2{
        y = 2;
    }
    return COLORS[x * 3 +  y] as f32;
}

fn decide_light(orig: f32, lighted: f32, noise: f32, is_noise_on: bool) -> f32{
    let value = orig*lighted + noise * (is_noise_on as i32) as f32;
    return clamp(value, 0.0, 1.0);
}

fn clinear_to_srgb(input: f32) -> f32{
    let a: f32 = 0.055;
    if input <= 0.0031308{
        return 12.92 * input;
    }
    else{
        let r1 = input.powf(1.0 / 2.4);
        let r2 = (1.0 + a) * r1;
        let r3 = r2 - a;
        return r3;
    }
}

fn srgb_to_clinear(input: usize) -> f32{
    let a: f32 = 0.055;
    let input_01 = (input as f32) / 255.0;
    if input < 11{
        return input_01 / 12.92;
    }
    else{
        let r1 = input_01 + a;
        let r2 = r1 / (1.0 + a);
        let r3 = r2.powf(2.4);
        return r3;
    }
}

fn generate_arr(sz: usize) -> ndarray::Array2::<f32>{
    let shape = (sz*3, sz*3);
    let mut arr = ndarray::Array2::<f32>::default(shape);
    ndarray::Zip::indexed(arr.outer_iter_mut()).par_for_each(|j, mut row| {
        for (k, col) in row.iter_mut().enumerate(){
            *col = decide(sz, j, k);
        }
    });
    return arr
}


fn find_min_max(arr: &ndarray::Array2::<f32>) -> (f32, f32){
    let shape =arr.shape();
    let mut min = arr[[0, 0]];
    let mut max = arr[[0, 0]];
    for i in 0..shape[0]{
        for j in 0..shape[1]{
            if arr[[i, j]] > max{
                max = arr[[i, j]];
            }
            if arr[[i, j]] < min{
                min = arr[[i, j]];
            }
        }
    }
    return (min, max);
}

fn prep_arr(arr: &ndarray::Array2::<f32>) -> ndarray::Array2::<f32>{
    let mut new_arr = ndarray::Array2::<f32>::default((arr.shape()[0], arr.shape()[1]));
    ndarray::Zip::indexed(new_arr.outer_iter_mut()).par_for_each(|j, mut row| {
        for (k, col) in row.iter_mut().enumerate(){
            *col = clinear_to_srgb(arr[[j, k]]) * 255.0;
        }
    });
    let (_min, max) = find_min_max(&new_arr);
    let coef = 255.0 / max;
    new_arr *= coef;
    return new_arr;
}

fn arr_to_img(arr: &ndarray::Array2::<f32>) -> image::GrayImage{
    let arr_ = prep_arr(&arr);
    let mut img = image::ImageBuffer::new(arr_.dim().0 as u32, arr_.dim().1 as u32);
    for (r, row) in arr_.outer_iter().enumerate()  {
        for (c, col) in row.iter().enumerate(){
            let pixel = (*col).round() as u8;
            let pixel = image::Luma([pixel]);
            img.put_pixel(r as u32, c as u32, pixel);
        }
    }
    return img;  
}

fn generate_arr_and_img(sz: usize)-> (ndarray::Array2::<f32>, image::GrayImage){
    let arr = generate_arr(sz);
    let img = arr_to_img(&arr);
    return (arr, img)
}


impl Scene{
    fn init(sz: usize) -> Self{
        let (arr, img) = generate_arr_and_img(sz);
        img.save("scene.png").unwrap();
        return Scene{
            scene_array: arr, 
            scene_image: img,
            size: sz
        };
    }

    fn recount_final_array(&self, light_matrix: &ndarray::Array2::<f32>, noise_matrix: &ndarray::Array2::<f32>, is_noise_on: bool) -> ndarray::Array2::<f32>{
        let shape = (self.size*3, self.size*3);
        let mut arr = ndarray::Array2::<f32>::default(shape);
        ndarray::Zip::indexed(arr.outer_iter_mut()).par_for_each(|j, mut row| {
            for (k, col) in row.iter_mut().enumerate(){
                *col = decide_light(self.scene_array[[j, k]], light_matrix[[j, k]], noise_matrix[[j, k]], is_noise_on);
            }
        });
        return arr;
    }

    fn update(&mut self, ls: &LightSource, ns: &Noise) -> ndarray::Array2::<f32>{
        let mut new_arr = self.recount_final_array(&ls.light_matrix, &ns.noise_array, ns.is_on);
        if !ls.is_on{
            new_arr = self.scene_array.clone();
        }
        self.scene_image = arr_to_img(&new_arr);
        self.scene_image.save("scene.png").unwrap();
        return new_arr;
    }
}

fn reverse_solve_task(path: &str){
    let img = image::open(path).unwrap().grayscale();
    let img = img.as_luma8().unwrap();
    let mut img_arr = ndarray::Array2::<f32>::default((img.width() as usize, img.height() as usize));
    for i in 0..900{
        for j in 0..900{
            img_arr[[i, j]] = srgb_to_clinear(img.get_pixel(i as u32, j as u32).0[0] as usize);
        }
    }
    let ls = LightSource::init(ALBEDO);
    let sc = Scene::init(300);
    let img_ = load_im_egui();
    let rev_sol_h = 0;
    let rev_sol_loc = (0, 0);
    let rev_sol_albed: Vec<f32> = [0.0, 0.0, 0.0, 
                                   0.0, 0.0, 0.0, 
                                   0.0, 0.0, 0.0].to_vec();
    let arr = img_arr;
    let ns = Noise::init();
    let mut lsa = LightSimApp { 
        light_source: ls, 
        scene: sc,
        noise: ns,
        reverse_solution_location: rev_sol_loc,
        img_gui: egui_extras::RetainedImage::from_color_image("sceneimg", img_),
        reverse_solution_height: rev_sol_h,
        revere_solution_albedo: rev_sol_albed,
        scene_arr: arr
    };
    lsa.update_no_pic();
    println!("height_sol: {}", lsa.reverse_solution_height as f32 / DIAG.sqrt());
    println!("loc_sol: {}", format!("{:?}", lsa.reverse_solution_location));
    println!("albedo_sol: {}", format!("{:?}", lsa.revere_solution_albedo));
}

//GUI
impl eframe::App for LightSimApp{
        fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame){
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            eframe::egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Light Simulation");
            ui.vertical(|ui|{
                ui.vertical(|ui|{
                    ui.add(eframe::egui::Slider::new(&mut self.light_source.height, 0..=1200).text("Light source height"));
                    eframe::egui::ComboBox::from_label("Light Position")
                    .selected_text(format!("{:?}", self.light_source.location)).show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.light_source.location, (0, 0), "(0, 0)");
                        ui.selectable_value(&mut self.light_source.location, (0, 1), "(0, 1)");
                        ui.selectable_value(&mut self.light_source.location, (0, 2), "(0, 2)");
                        ui.selectable_value(&mut self.light_source.location, (1, 0), "(1, 0)");
                        ui.selectable_value(&mut self.light_source.location, (1, 1), "(1, 1)");
                        ui.selectable_value(&mut self.light_source.location, (1, 2), "(1, 2)");
                        ui.selectable_value(&mut self.light_source.location, (2, 0), "(2, 0)");
                        ui.selectable_value(&mut self.light_source.location, (2, 1), "(2, 1)");
                        ui.selectable_value(&mut self.light_source.location, (2, 2), "(2, 2)");
                    });
                    ui.add(eframe::egui::Slider::new(&mut self.light_source.coordinates.0, -1200..=(SIZE-1) as i32).text("Light source X coordinate"));
                    ui.add(eframe::egui::Slider::new(&mut self.light_source.coordinates.1, -1200..=(SIZE-1) as i32).text("Light source Y coordinate"));
                    ui.add(eframe::egui::Checkbox::new(&mut self.light_source.is_on, "Turn the light on"));
                    ui.add(eframe::egui::Checkbox::new(&mut self.noise.is_on, "Noise"));
                });
            self.img_gui.show(ui);
            if ui.button("Save pic").clicked(){
                let loc = get_actual_location(self.light_source.coordinates, self.light_source.location, SIZE);

                let mut path = "scene_x".to_string() + &loc.0.to_string() + "_y" + &loc.1.to_string()+ "_h" + self.light_source.height.to_string().as_str();
                if self.noise.is_on{
                    path += &"_noised".to_string();
                }
                path += &".png".to_string();
                self.scene.scene_image.save(path).unwrap();
            }
            if ui.button("Load pic").clicked(){
                let path = "mondrian_albedo_estimation_frame_3.png";
                reverse_solve_task(path);
            }
            if self.light_source.is_on{
                self.update_();
                ui.label("Reverse task soltions:");
                ui.horizontal(|ui| {
                    ui.label(format!("location: {:?}", self.reverse_solution_location));
                    ui.label(format!(" ~ error{}", (eucl_dist(&get_actual_location(self.light_source.coordinates, self.light_source.location, SIZE), &self.reverse_solution_location) as f32 / DIAG.sqrt())));
                 });
                ui.horizontal(|ui| {
                    ui.label(format!("height: {}", (self.reverse_solution_height as f32 / DIAG.sqrt() as f32)));
                    ui.label(format!(" ~ error {}", (self.reverse_solution_height.abs_diff(self.light_source.height) as f32/ self.light_source.height as f32)));
                });
                for i in 0..3{
                    for j in 0..3{
                        ui.horizontal(|ui| {
                            ui.label(format!("Albedo [{}, {}] / Maximum Albedo: {:.2}", i, j, self.revere_solution_albedo[3*i + j]));
                            ui.label(format!(" ~ error {:.3}", (self.revere_solution_albedo[3*i + j] - (ALBEDO[3*i + j] / ALBEDO.iter().max_by(|p, l| p.partial_cmp(l).unwrap()).unwrap())).abs()));
                        });

                    }
                }
            }
            });
            });
        if ui.button("Quit").clicked(){
            _frame.close();
        }
        });
    }    
}

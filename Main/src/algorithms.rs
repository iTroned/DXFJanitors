
use dxfextract::PolyLine;

use log::{error, info, warn};
use std::{collections::{HashMap, BTreeMap}, f64::consts::PI, vec, fmt};

use crate::dxfextract;

const SMALLEST_ANGLE: f64 = 5.;
#[derive(Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

//implementation of init, clone and angle between this and another point
impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self {x, y}
    }
    pub fn clone(&self) -> Self { 
        *self
    }
    pub fn angle_to(&self, other: &Point) -> f64 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let angle = dy.atan2(dx);
        if angle < 0.0 {
            angle + 2.0 * PI
        } else {
            angle
        }
    }
}
//to_string implementation
impl fmt::Display for Point {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("(");
        fmt.write_str(&self.x.to_string());
        fmt.write_str(", ");
        fmt.write_str(&self.y.to_string());
        fmt.write_str(")");
        Ok(())
    }
}
#[derive(Clone, Copy, PartialEq)]
pub struct PointWithNeighbour {
    pub point: Point,
    pub neighbour: Point
}
/*impl PointWithNeighbour {
    pub fn new(point: CustomPoint, neighbour: CustomPoint) -> Self {
        Self {point, neighbour}
    }
    pub fn clone(&self) -> Self {
        *self
    }
}*/
//first try on finding intersection
/*fn intersection_of_points(a1: &CustomPoint, a2: &CustomPoint, b1: &CustomPoint, b2: &CustomPoint) -> CustomPoint{
    //y = mx + b
    let a_m = a2.y - a1.y / a2.x - a1.x;
    let a_b = a1.y - a_m * a1.x;
    let b_m = b2.y - b1.y / b2.x - b1.x;
    let b_b = b1.y - b_m * b1.x;

    //a_m * x + a_b = b_m * x + b_b

    let x = (b_b - a_b) / (a_m - b_m);
    let y = a_m * x + a_b;
    let new_point = CustomPoint::new(x, y);
    new_point
}*/
//second try on finding intersection. partially generated by ChatGPT
fn intersection(p1: &Point, p2: &Point, p3: &Point, p4: &Point) -> Option<Point> {
    let x1 = p1.x;
    let y1 = p1.y;
    let x2 = p2.x;
    let y2 = p2.y;
    let x3 = p3.x;
    let y3 = p3.y;
    let x4 = p4.x;
    let y4 = p4.y;

    //denominator
    let denom = (y4 - y3) * (x2 - x1) - (x4 - x3) * (y2 - y1);
    if denom == 0.0 {
        return None;
    }

    let ua = ((x4 - x3) * (y1 - y3) - (y4 - y3) * (x1 - x3)) / denom;
    let _ub = ((x2 - x1) * (y1 - y3) - (y2 - y1) * (x1 - x3)) / denom;

    let x = x1 + ua * (x2 - x1);
    let y = y1 + ua * (y2 - y1);

    //reterns the position of the intersection
    Some(Point::new(x, y))
}
/*fn _find_closest_point(point: CustomPoint, vector: &Vec<PointWithNeighbour>) -> &PointWithNeighbour{
    let mut closest_point = vector.first().to_owned().unwrap();
    let mut closest_distance = f64::sqrt((closest_point.point.x - point.x) * (closest_point.point.x - point.x) + (closest_point.point.y - point.y) * (closest_point.point.y - point.y));
    for v_point in vector{
        let new_distance = f64::sqrt((v_point.point.x - point.x) * (v_point.point.x - point.x) + (v_point.point.y - point.y) * (v_point.point.y - point.y));
        if new_distance == 0. {
            continue;
        }
        if new_distance < closest_distance || closest_distance == 0. {
            closest_distance = new_distance;
            closest_point = v_point;
        }
    }
    closest_point
}*/
//returns the distance between two points
fn distance(point_1x: &f64, point_1y: &f64, point_2x: &f64, point_2y: &f64) -> f64{
    f64::sqrt((point_1x - point_2x) * (point_1x - point_2x) + (point_1y - point_2y) * (point_1y - point_2y))
}

//angle at the point two linear functions intercept
//angle for two linear lines: angle = tan^-1 (|m2-m1|/(1+m1m2)) Where m1 is the slope of function A and m2 is the slope of function B
fn angle_between_lines(m1: f64, m2: f64) -> f64{
    let angle = ((m2-m1).abs()/(1.0+m1*m2)).atan();
    angle * 180. / PI
}

//angle between vectors 
//angle = arccos((a*b)/|a||b|) -> where a*b is the dot product and |a| and |b| is the length of the vectors
fn angle_vectors(v1: (f64, f64), v2: (f64, f64)) -> f64{
    //in a tuple the values are v1.0 and v1.1
    let length_v1 = ((v1.0 * v1.0) + (v1.1*v1.1)).sqrt(); //the length of a vector is |u| = sqrt(x^2+y^2)
    let length_v2 = ((v2.0*v2.0) + (v2.1*v2.1)).sqrt();

    let dotproduct = (v1.0*v2.0) + (v1.1*v2.1); //dot product of a 2D vector u*v = x1x2 + y1y2

    let angle = (dotproduct/(length_v1 * length_v2)).acos();
    angle * 180. / PI //return angle in degrees (f64)
}

//B is the vertex where the angle is calculated
//function creates two vectors and uses the function angle vectors to return the angle
fn angle_three_points(a: Point, b: Point, c: Point) -> f64{
    //creating vectors: AB and BC
    /*let ab = (a.0 - b.0, b.1 - a.1); //vector AB = (B1 - A1, B2 - A2)
    let bc = (c.0 - b.0, c.1 - b.1);

    let angle = angle_vectors(ab, bc);
    angle //return angle*/
    let angle_degrees = (180.0 / PI) * (b.angle_to(&a) - b.angle_to(&c)).abs();
    angle_degrees
}
//first iteration we tried. ended op not working due to f64 not implementing eq
/*fn _extend_closest_lines(in_map: &HashMap<String, Vec<PolyLine>>) -> HashMap<String, Vec<PolyLine>>{
    let mut out_map = HashMap::<String, Vec<PolyLine>>::default();
    for (name, in_polylines) in in_map{
        let mut out_polylines = Vec::<PolyLine>::default();
        let mut xy_ends: Vec<PointWithNeighbour> = Vec::new();
        let mut buddy_map = HashMap::<PointWithNeighbour, PolyLine>::default();
        for polyline in in_polylines {
            if polyline.is_closed {
                out_polylines.push(polyline.clone());
                continue;
            }

            let len: i32 = polyline.x_values.len().try_into().unwrap();
            let mut x_val = polyline.x_values.clone();
            let mut y_val = polyline.y_values.clone();
            if len == 2 {
                let point = PointWithNeighbour::new(CustomPoint::new(match polyline.x_values.first() {
                    None => 0.0,
                    Some(x) => x.clone(),
                }, match polyline.y_values.first() {
                    None => 0.0,
                    Some(x) => x.clone(),
                }), CustomPoint::new(match polyline.x_values.last() {
                    None => 0.0,
                    Some(x) => x.clone(),
                }, match polyline.y_values.last() {
                    None => 0.0,
                    Some(x) => x.clone(),
                }));
                //buddy_map.insert(point, polyline);
                xy_ends.push(point);
                let point_2 = PointWithNeighbour::new(CustomPoint::new(match polyline.x_values.last() {
                    None => 0.0,
                    Some(x) => x.clone(),
                }, match polyline.y_values.last() {
                    None => 0.0,
                    Some(x) => x.clone(),
                }), CustomPoint::new(match polyline.x_values.first() {
                    None => 0.0,
                    Some(x) => x.clone(),
                }, match polyline.y_values.first() {
                    None => 0.0,
                    Some(x) => x.clone(),
                }));
                xy_ends.push(point_2);
            }
            else if len == 3 {
                let x1 = match x_val.pop(){
                    None => 0.0,
                    Some(x) => x,
                };
                let x2 = match x_val.pop(){
                    None => 0.0,
                    Some(x) => x,
                };
                let x3 = match x_val.pop(){
                    None => 0.0,
                    Some(x) => x,
                };
                let y1 = match y_val.pop(){
                    None => 0.0,
                    Some(x) => x,
                };
                let y2 = match y_val.pop(){
                    None => 0.0,
                    Some(x) => x,
                };
                let y3 = match y_val.pop(){
                    None => 0.0,
                    Some(x) => x,
                };
                let mid_point = CustomPoint::new(x2, y2);
                xy_ends.push(PointWithNeighbour::new(CustomPoint::new(x1, y1), mid_point.clone()));
                xy_ends.push(PointWithNeighbour::new(CustomPoint::new(x3, y3), mid_point.clone()));
            }
            else{
                //Adds the last coordinates to the vector
                xy_ends.push(PointWithNeighbour::new(CustomPoint::new(x_val.pop().unwrap(), y_val.pop().unwrap()), CustomPoint::new(x_val.pop().unwrap(), y_val.pop().unwrap())));
                let mut i = len - 4;
                while i > 0 {
                    x_val.pop();
                    y_val.pop();
                    i -= 1;
                }
                let x1 = match x_val.pop(){
                    None => 0.0,
                    Some(x) => x,
                };
                let y1 = match y_val.pop(){
                    None => 0.0,
                    Some(x) => x,
                };
                xy_ends.push(PointWithNeighbour::new(CustomPoint::new(match x_val.pop(){
                    None => 0.0,
                    Some(x) => x,
                }, match y_val.pop(){
                    None => 0.0,
                    Some(x) => x,
                }), CustomPoint::new(x1, y1)));
            }

        }
        for point in xy_ends.clone(){
            let closest = find_closest_point(point.point, &xy_ends);
            let closest_to_closest = find_closest_point(closest.point, &xy_ends);
            if !point.eq(closest_to_closest) {
                //out_polylines.push
                continue;
            }
            let common_point = interception_of_points(&point.point, &point.neighbour, &closest.point, &closest.neighbour);
            //out_polylines.push(PolyLine::new(false, ));
        }
        out_map.insert(name.clone(), out_polylines);
    }
    out_map
}*/
//function that connects polylines inside of each layer depending on the parameters given. O(n^2) so needs optimizing later down the line
pub fn try_to_close_polylines(extend: bool, all_layers: &BTreeMap<String, Vec<PolyLine>>, affected_layers: &BTreeMap<String, Vec<PolyLine>>, max_distance_in: &Option<f64>, max_angle_in: &Option<i32>, o_iterations: &Option<i32>) -> BTreeMap<String, Vec<PolyLine>> {
    let mut out = all_layers.clone();
    let mut iterations;
    let mut done_iterations = 0;
    let mut any_changes = true;
    let max_distance;
    //currently not used, but may have use for limiting angles
    let _max_angle;
    if let Some(amount) = o_iterations.clone(){
        iterations = amount;
    }
    else{
        iterations = 1;
    }
    if let Some(angle) = max_angle_in.clone(){
        _max_angle = angle;
    }
    else{
        _max_angle = 180;
    }
    if let Some(distance) = max_distance_in.clone(){
        max_distance = distance;
    } 
    else {
        max_distance = 0.0;
    }
    //stops if max iterations has been reached, or there were no changes in the previous iteration
    while &iterations > &0 && any_changes {
        iterations -= 1;
        done_iterations += 1;
        any_changes = false;
        let mut current_map = BTreeMap::<String, Vec<PolyLine>>::new();
        for(name, polylines) in &out{
            if !affected_layers.contains_key(name){
                continue;
            }
            let mut out_polylines = Vec::<PolyLine>::default();
            let mut iter = polylines.clone();
            let mut has_changed = Vec::<PolyLine>::default();
            
            while let Some(mut polyline) = iter.pop(){
                if has_changed.contains(&polyline){
                    continue;
                }
                if polyline.is_closed{
                    out_polylines.push(polyline);
                    continue;
                }
                let start_x = polyline.x_values.first().unwrap();
                let start_y = polyline.y_values.first().unwrap();
                let end_x = polyline.x_values.last().unwrap();
                let end_y = polyline.y_values.last().unwrap();
                let start_distance = distance(start_x, start_y, end_x, end_y);
                let mut min_distance_start = start_distance.clone();
                let mut min_distance_end = start_distance.clone();
                let mut should_close = true;
                let mut start_connection = None;
                let mut end_connection = None;
            
                //handles different edge cases. literally
                let mut start_is_start = false;
                let mut end_is_start = false;
                //iterates through the polylines in the collection. skips itself as well as closed polylines
                for cmp_polyline in polylines{
                    //doesnt interact with closed polylines nor itself
                    if cmp_polyline.is_closed || cmp_polyline.clone() == polyline.clone(){
                        continue;
                    }
                    
                    let cmp_start_x = cmp_polyline.x_values.first().unwrap();
                    let cmp_start_y = cmp_polyline.y_values.first().unwrap();
                    let cmp_end_x = cmp_polyline.x_values.last().unwrap();
                    let cmp_end_y = cmp_polyline.y_values.last().unwrap();
                    
                    //checks startpoint of selected polyline
                    //against startpoint of current
                    let mut cur_distance = distance(start_x, start_y, cmp_start_x, cmp_start_y);
                    if cur_distance < min_distance_start && cur_distance <= max_distance{
                        min_distance_start = cur_distance;
                        start_connection = Some(cmp_polyline);
                        start_is_start = true;
                    }
                    //against endpoint of current
                    cur_distance = distance(start_x, start_y, cmp_end_x, cmp_end_y);
                    if cur_distance < min_distance_start && cur_distance <= max_distance{
                        
                        min_distance_start = cur_distance;
                        start_connection = Some(cmp_polyline);
                        start_is_start = false;
                    }
                    //min_distance = start_distance;
    
                    //checks endpoint of selected polyline
                    //against startpoint of current
                    cur_distance = distance(end_x, end_y, cmp_start_x, cmp_start_y);
                    if cur_distance < min_distance_end && cur_distance <= max_distance{
                        
                        min_distance_end = cur_distance;
                        end_connection = Some(cmp_polyline);
                        end_is_start = true;
                    }
                    //against endpoint of current
                    cur_distance = distance(end_x, end_y, cmp_end_x, cmp_end_y);
                    if cur_distance < min_distance_end && cur_distance <= max_distance{
                        
                        min_distance_end = cur_distance;
                        end_connection = Some(cmp_polyline);
                        end_is_start = false;
                    }
                }
                //skips cases where they are not each others closest // hard to make it optimal // need to come up with something better
                /* 
                if let Some(remove_start) = start_connection {
                    let mut min_distance = i32::MAX;
                    for cmp_polyline in polylines {
                        if cmp_polyline.is_closed || cmp_polyline.clone() == remove_start.clone(){
                            continue;
                        }
                        let cmp_start_x = cmp_polyline.x_values.first().unwrap();
                        let cmp_start_y = cmp_polyline.y_values.first().unwrap();
                        let cmp_end_x = cmp_polyline.x_values.last().unwrap();
                        let cmp_end_y = cmp_polyline.y_values.last().unwrap();
                        let cur_distance = distance(end_x, end_y, cmp_end_x, cmp_end_y);
                    }
                }*/
                
                if let (Some(remove_start), Some(remove_end)) = (start_connection, end_connection) {
                    should_close = false;
                    any_changes = true;
                    //println!("case1");
                    if has_changed.contains(remove_end) || has_changed.contains(remove_start) {
                        out_polylines.push(polyline.clone());
                        continue;
                    }
                    
                    has_changed.push(remove_start.clone());
                    has_changed.push(remove_end.clone());
                    if extend {
                        let mut last_point_1;
                        let mut last_point_2;
                        let mut second_last_point_1;
                        let mut second_last_point_2;
                        let mut new_x_values;
                        let mut new_y_values;
                        if start_is_start {
                            new_x_values = reverse_vector(remove_start.x_values.clone());
                            new_y_values = reverse_vector(remove_start.y_values.clone());
                            last_point_1 = Point::new(new_x_values.pop().unwrap(), new_y_values.pop().unwrap());
                            second_last_point_1 = Point::new(new_x_values.last().unwrap().clone(), new_y_values.last().unwrap().clone());
                        }
                        else{
                            new_x_values = remove_start.x_values.clone();
                            new_y_values = remove_start.y_values.clone();
                            last_point_1 = Point::new(new_x_values.pop().unwrap(), new_y_values.pop().unwrap());
                            second_last_point_1 = Point::new(new_x_values.last().unwrap().clone(), new_y_values.last().unwrap().clone());
                        }
                        let mut polyline_values_x = reverse_vector(polyline.x_values.clone());
                        let mut polyline_values_y = reverse_vector(polyline.y_values.clone());
                        last_point_2 = Point::new(polyline_values_x.pop().unwrap(), polyline_values_y.pop().unwrap());
                        second_last_point_2 = Point::new(polyline_values_x.last().unwrap().clone(), polyline_values_y.last().unwrap().clone());
                        let mut interception = Point::new(0., 0.);
                        if let Some(found_point) = intersection(&last_point_1, &second_last_point_1, &last_point_2, &second_last_point_2){
                            interception = found_point;
                        }
                        /*let angle = angle_three_points(last_point_1, interception, last_point_2);
                        println!("{}", angle);*/
                        new_x_values.push(interception.x);
                        new_y_values.push(interception.y);
                        new_x_values.append(&mut reverse_vector(polyline_values_x));
                        new_y_values.append(&mut reverse_vector(polyline_values_y));
                        last_point_1 = Point::new(new_x_values.pop().unwrap(), new_y_values.pop().unwrap());
                        second_last_point_1 = Point::new(new_x_values.last().unwrap().clone(), new_y_values.last().unwrap().clone());
                        let mut remove_x_values = remove_end.x_values.clone();
                        let mut remove_y_values = remove_end.y_values.clone();
                        if end_is_start {
                            remove_x_values = reverse_vector(remove_x_values);
                            remove_y_values = reverse_vector(remove_y_values);
                            last_point_2 = Point::new(remove_x_values.pop().unwrap(), remove_y_values.pop().unwrap());
                            second_last_point_2 = Point::new(remove_x_values.last().unwrap().clone(), remove_y_values.last().unwrap().clone());
                            remove_x_values = reverse_vector(remove_x_values);
                            remove_y_values = reverse_vector(remove_y_values);
                        }
                        else{
                            last_point_2 = Point::new(remove_x_values.pop().unwrap(), remove_y_values.pop().unwrap());
                            second_last_point_2 = Point::new(remove_x_values.last().unwrap().clone(), remove_y_values.last().unwrap().clone());
                            remove_x_values = reverse_vector(remove_x_values);
                            remove_y_values = reverse_vector(remove_y_values);
                        }
    
                        if let Some(found_point) = intersection(&last_point_1, &second_last_point_1, &last_point_2, &second_last_point_2){
                            interception = found_point;
                        }
                        /*let angle = angle_three_points(last_point_1, interception, last_point_2);
                        println!("{}", angle);*/
                        //interception = (interception_of_points(&last_point_1, &second_last_point_1, &last_point_2, &second_last_point_2));
                        new_x_values.push(interception.x);
                        new_y_values.push(interception.y);
                        let closed = remove_start == remove_end;
                        if !closed {
                            new_x_values.append(&mut remove_x_values);
                            new_y_values.append(&mut remove_y_values);
                        }
                        
                        
                        //println!("Case1 X-length: {}, Y-length: {}", new_x_values.len(), new_y_values.len());
                        out_polylines.push(PolyLine::new(closed, new_x_values, new_y_values));
                    }
                    else{
                        let mut new_x_values;
                        let mut new_y_values;
                        if start_is_start {
                            new_x_values = reverse_vector(remove_start.x_values.clone());
                            new_y_values = reverse_vector(remove_start.y_values.clone());
                        }
                        else{
                            new_x_values = remove_start.x_values.clone();
                            new_y_values = remove_start.y_values.clone();
                        }
                        new_x_values.append(&mut polyline.x_values.clone());
                        new_y_values.append(&mut polyline.y_values.clone());
                        
                        let closed = remove_start == remove_end;
                        if !closed{
                            if end_is_start {
                                new_x_values.append(&mut remove_end.x_values.clone());
                                new_y_values.append(&mut remove_end.y_values.clone());
                            }
                            else{
                                new_x_values.append(&mut reverse_vector(remove_end.x_values.clone()));
                                new_y_values.append(&mut reverse_vector(remove_end.y_values.clone()));
                            }
                        }
                        out_polylines.push(PolyLine::new(closed, new_x_values, new_y_values));
                    }
                    
                }
                
                else if let Some(remove) = start_connection {
                    should_close = false;
                    any_changes = true;
                    //println!("case2");
                    //waits for next iteration
                    if has_changed.contains(remove){
                        out_polylines.push(polyline.clone());
                        continue;
                    }
                    has_changed.push(remove.clone());
                    if extend {
                        let last_point_1;
                        let last_point_2;
                        let second_last_point_1;
                        let second_last_point_2;
                        let mut new_x_values;
                        let mut new_y_values;
                        if start_is_start {
                            new_x_values = reverse_vector(remove.x_values.clone());
                            new_y_values = reverse_vector(remove.y_values.clone());
                            last_point_1 = Point::new(new_x_values.pop().unwrap(), new_y_values.pop().unwrap());
                            second_last_point_1 = Point::new(new_x_values.last().unwrap().clone(), new_y_values.last().unwrap().clone());
                        }
                        else{
                            new_x_values = remove.x_values.clone();
                            new_y_values = remove.y_values.clone();
                            last_point_1 = Point::new(new_x_values.pop().unwrap(), new_y_values.pop().unwrap());
                            second_last_point_1 = Point::new(new_x_values.last().unwrap().clone(), new_y_values.last().unwrap().clone());
                        }
                        let mut polyline_values_x = reverse_vector(polyline.x_values.clone());
                        let mut polyline_values_y = reverse_vector(polyline.y_values.clone());
                        last_point_2 = Point::new(polyline_values_x.pop().unwrap(), polyline_values_y.pop().unwrap());
                        second_last_point_2 = Point::new(polyline_values_x.last().unwrap().clone(), polyline_values_y.last().unwrap().clone());
                        let mut interception = Point::new(0., 0.);
                        if let Some(found_point) = intersection(&last_point_1, &second_last_point_1, &last_point_2, &second_last_point_2){
                            interception = found_point;
                        }
                        /*let angle = angle_three_points(last_point_1, interception, last_point_2);
                        println!("{}", angle);*/
                        new_x_values.push(interception.x);
                        new_y_values.push(interception.y);
                        new_x_values.append(&mut reverse_vector(polyline_values_x));
                        new_y_values.append(&mut reverse_vector(polyline_values_y));
                        //println!("Case2 X-length: {}, Y-length: {}", new_x_values.len(), new_y_values.len());
                        out_polylines.push(PolyLine::new(false, new_x_values, new_y_values));
                    }
                    else {
                        if start_is_start{
                            let mut new_x_values = reverse_vector(polyline.x_values.clone());
                            new_x_values.append(&mut remove.x_values.clone());
                            let mut new_y_values = reverse_vector(polyline.y_values.clone());
                            new_y_values.append(&mut remove.y_values.clone());
                            
                            out_polylines.push(PolyLine::new(false, new_x_values, new_y_values));
                        }
                        else{
                            let mut new_x_values = reverse_vector(polyline.x_values.clone());
                            new_x_values.append(&mut reverse_vector(remove.x_values.clone()));
                            let mut new_y_values = reverse_vector(polyline.y_values.clone());
                            new_y_values.append(&mut reverse_vector(remove.y_values.clone()));
                            
                            out_polylines.push(PolyLine::new(false, new_x_values, new_y_values));
                        }
                    }
                    
                }
                else if let Some(remove) = end_connection {
                    //skips this connection if the connector already has been used this iteration
                    //println!("case3");
                   
                    
                    should_close = false;
                    any_changes = true;
                    if has_changed.contains(remove){
                        out_polylines.push(polyline.clone());
                        continue;
                    }
                    has_changed.push(remove.clone());
                    if extend {
                        let last_point_1;
                        let last_point_2;
                        let second_last_point_1;
                        let second_last_point_2;
                        let mut new_x_values = polyline.x_values.clone();
                        let mut new_y_values = polyline.y_values.clone();
                        last_point_1 = Point::new(new_x_values.pop().unwrap(), new_y_values.pop().unwrap());
                        second_last_point_1 = Point::new(new_x_values.last().unwrap().clone(), new_y_values.last().unwrap().clone());
                        let mut remove_x_values = remove.x_values.clone();
                        let mut remove_y_values = remove.y_values.clone();
                        if end_is_start {
                            remove_x_values = reverse_vector(remove_x_values);
                            remove_y_values = reverse_vector(remove_y_values);
                            last_point_2 = Point::new(remove_x_values.pop().unwrap(), remove_y_values.pop().unwrap());
                            second_last_point_2 = Point::new(remove_x_values.last().unwrap().clone(), remove_y_values.last().unwrap().clone());
                            remove_x_values = reverse_vector(remove_x_values);
                            remove_y_values = reverse_vector(remove_y_values);
                        }
                        else{
                            last_point_2 = Point::new(remove_x_values.pop().unwrap(), remove_y_values.pop().unwrap());
                            second_last_point_2 = Point::new(remove_x_values.last().unwrap().clone(), remove_y_values.last().unwrap().clone());
                            remove_x_values = reverse_vector(remove_x_values);
                            remove_y_values = reverse_vector(remove_y_values);
                        }
                        let mut interception = Point::new(0., 0.);
                        if let Some(found_point) = intersection(&last_point_1, &second_last_point_1, &last_point_2, &second_last_point_2){
                            interception = found_point;
                        }
                        /*let angle = angle_three_points(last_point_1, interception, last_point_2);
                        println!("{}", angle);*/
                        //interception = (interception_of_points(&last_point_1, &second_last_point_1, &last_point_2, &second_last_point_2));
                        new_x_values.push(interception.x);
                        new_y_values.push(interception.y);
                        new_x_values.append(&mut remove_x_values);
                        new_y_values.append(&mut remove_y_values);
                        //println!("Case3 X-length: {}, Y-length: {}", new_x_values.len(), new_y_values.len());
                        out_polylines.push(PolyLine::new(false, new_x_values, new_y_values));
                    }
                    else {
                        if end_is_start{
                            let mut new_x_values = polyline.x_values.clone();
                            new_x_values.append(&mut remove.x_values.clone());
                            let mut new_y_values = polyline.y_values.clone();
                            new_y_values.append(&mut remove.y_values.clone());
                            
                            out_polylines.push(PolyLine::new(false, new_x_values, new_y_values));
                        }
                        else{
                            let mut new_x_values = polyline.x_values.clone();
                            new_x_values.append(&mut reverse_vector(remove.x_values.clone()));
                            let mut new_y_values = polyline.y_values.clone();
                            new_y_values.append(&mut reverse_vector(remove.y_values.clone()));
                            
                            out_polylines.push(PolyLine::new(false, new_x_values, new_y_values));
                        }
                    }
                    
                }
    
                //If the closest point is part of the same polyline
                if should_close {
                    //println!("closing {}, {}, {}", polyline.x_values.len(), start_distance, max_distance);
                    polyline.is_closed = polyline.x_values.len() != 2 && start_distance <= max_distance;
                    //polyline.x_values.pop();
                    //polyline.y_values.pop();
                    out_polylines.push(polyline);
                    //println!("\n closest point is part of same polyline..");
                    //out_polylines.push(PolyLine::new(true, polyline.x_values.clone(), polyline.y_values.clone()));
                    continue;
                }
                
            }
            current_map.insert(name.clone(), out_polylines);
        }
        info!("Iterations executed: {}, Type: Connect", done_iterations);
        for (name, layer) in all_layers {
            if affected_layers.contains_key(name){
                continue;
            }
            current_map.insert(name.clone(), layer.clone());
        }
        out = current_map;
    }
out
}
//finds and returns the min and max x and y for a given map
pub fn calculate_min_max(layer_polylines: &BTreeMap<String, Vec<PolyLine>>) -> Option<(f64, f64, f64, f64, f64)>{
    let all_polylines: Vec<PolyLine> = layer_polylines
        .values()
        .flat_map(|v| v.iter().cloned())
        .collect();

    // compute stats for polylines
    let x_values: Vec<f64> = all_polylines
        .iter()
        .flat_map(|e| e.x_values.clone())
        .collect();

    let y_values: Vec<f64> = all_polylines
        .iter()
        .flat_map(|e| e.y_values.clone())
        .collect();

    let cmp = |a: &f64, b: &f64| f64::partial_cmp(a, b).unwrap();
    let min_x = x_values.iter().copied().min_by(cmp).unwrap();
    let max_x = x_values.iter().copied().max_by(cmp).unwrap();
    let min_y = y_values.iter().copied().min_by(cmp).unwrap();
    let max_y = y_values.iter().copied().max_by(cmp).unwrap();

    // create document
    let width = max_x - min_x;
    let height = max_y - min_y;
    Some((min_x, min_y, max_y, width, height))
}

//creates a flipped copy of a given vector - ineffecient, should be flagged
pub fn reverse_vector(mut vector: Vec<f64>) -> Vec<f64>{
    let mut out = Vec::<f64>::default();
    while let Some(val) = vector.pop(){
        out.push(val);
    }
    out
}

use dxfextract::PolyLine;


use std::{collections::HashMap, f64::consts::PI};

use crate::dxfextract;
#[derive(Clone, Copy, PartialEq)]
pub struct CustomPoint {
    pub x: f64,
    pub y: f64,
}

impl CustomPoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self {x, y}
    }
    pub fn clone(&self) -> Self { 
        *self
    }
}
#[derive(Clone, Copy, PartialEq)]
pub struct PointWithNeighbour {
    pub point: CustomPoint,
    pub neighbour: CustomPoint
}
impl PointWithNeighbour {
    pub fn new(point: CustomPoint, neighbour: CustomPoint) -> Self {
        Self {point, neighbour}
    }
    pub fn clone(&self) -> Self {
        *self
    }
}

fn connect_points(a1: &CustomPoint, a2: &CustomPoint, b1: &CustomPoint, b2: &CustomPoint) -> CustomPoint{
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
}
fn find_closest_point(point: CustomPoint, vector: &Vec<PointWithNeighbour>) -> &PointWithNeighbour{
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
fn angle_three_points(a: (f64, f64), b: (f64, f64), c: (f64, f64)) -> f64{
    //creating vectors: AB and BC
    let ab = (a.0 - b.0, b.1 - a.1); //vector AB = (B1 - A1, B2 - A2)
    let bc = (c.0 - b.0, c.1 - b.1);

    let angle = angle_vectors(ab, bc);
    angle //return angle
}

fn extend_closest_lines(in_map: &HashMap<String, Vec<PolyLine>>) -> HashMap<String, Vec<PolyLine>>{
    let mut out_map = HashMap::<String, Vec<PolyLine>>::default();
    for (name, in_polylines) in in_map{
        let mut out_polylines = Vec::<PolyLine>::default();
        let mut xy_ends: Vec<PointWithNeighbour> = Vec::new();
        let mut buddy_map = HashMap::<PointWithNeighbour, PolyLine>::default();
        for polyline in in_polylines {
            if polyline.is_closed {
                out_polylines.push(polyline.clone());
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
            let common_point = connect_points(&point.point, &point.neighbour, &closest.point, &closest.neighbour);
            //out_polylines.push(PolyLine::new(false, ));
        }
        out_map.insert(name.clone(), out_polylines);
    }
    out_map
}
pub fn calculate_min_max(layer_polylines: &HashMap<String, Vec<PolyLine>>) -> (f64, f64, f64, f64, f64){
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
    (min_x, min_y, max_y, width, height)
}
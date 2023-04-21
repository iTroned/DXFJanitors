#[cfg(test)]

use dxf_janitors;
use dxf_janitors::dxfextract::PolyLine;
use crate::dxf_janitors::algorithms;
use std::{collections::{HashMap, BTreeMap}, f64::consts::PI, vec, fmt};

//Integration tests => test public functions that are in use in other modules
//Connect and Extend is from algorithm.rs used by main.rs
//This test is testing the PolyLine struct from dxfextract.rs

#[test]
fn test_try_to_extend_polylines(){    
    //Create a Sample data set
        //BTreeMap
        let mut test_layers: BTreeMap<String, Vec<PolyLine>> = BTreeMap::new();
        let mut test_affected_layers: BTreeMap<String, Vec<PolyLine>> = BTreeMap::new();

        //Random line
        let x1_values = vec![1.0, 2.0, 2.0, 2.0, 1.0]; 
        let y1_values = vec![1.0, 1.0, 2.0, 3.0, 3.0];
        let polyline1 = PolyLine::new(false, x1_values, y1_values);

        //Start is start
        let x2_values = vec![1.0, 2.0, 3.0]; 
        let y2_values = vec![1.0, 1.0, 1.0];
        let start_is_start = PolyLine::new(false, x2_values, y2_values);

        //Start is end
        let x5_values = vec![3.0, 2.0, 1.0]; 
        let y5_values = vec![1.0, 1.0, 1.0];
        let start_is_end = PolyLine::new(false, x5_values, y5_values);

        //End is start
        let x3_values = vec![4.0, 5.0, 6.0]; 
        let y3_values = vec![2.0, 3.0, 3.0];
        let end_is_start = PolyLine::new(false, x3_values, y3_values);

        //End is end
        let x4_values = vec![6.0, 5.0, 4.0]; 
        let y4_values = vec![3.0, 3.0, 2.0];
        let end_is_end = PolyLine::new(false, x4_values, y4_values);

        let mut expected: BTreeMap<String, Vec<PolyLine>> = BTreeMap::new();

        //Test case 0: Only 1 iteration => line should give correct values, but is_closed = false
        test_layers.insert(String::from("test0"), vec![start_is_start.clone(), end_is_start.clone()]);
        test_affected_layers.insert(String::from("test0"), vec![start_is_start.clone(), end_is_start.clone()]);
        let result = algorithms::try_to_close_polylines(true, &test_layers, &test_affected_layers, &Some(100.0), &Some(180), &Some(1));

        let x1_values = vec![1.0, 2.0, 3.0, 5.0, 6.0]; 
        let y1_values = vec![1.0, 1.0, 1.0, 3.0, 3.0];
        let polyline3 = PolyLine::new(false, x1_values, y1_values);
        expected.insert(String::from("test0"), vec![polyline3.clone()]);

        assert_eq!(result, expected);

        
        //remove the data from this test so it do not interfere with later tests
        test_layers.remove("test0");
        test_affected_layers.remove("test0");
        expected.remove("test0");


        //Test case 1: Close with it's own polyline => is closed from false to true
        test_layers.insert(String::from("test1"), vec![polyline1.clone()]);
        test_affected_layers.insert(String::from("test1"), vec![polyline1.clone()]);
        let result = algorithms::try_to_close_polylines(true, &test_layers, &test_affected_layers, &Some(100.0), &Some(180), &Some(10));


        let x1_values = vec![1.0, 2.0, 2.0, 2.0, 1.0]; 
        let y1_values = vec![1.0, 1.0, 2.0, 3.0, 3.0];
        let polyline1 = PolyLine::new(true, x1_values, y1_values);
        expected.insert(String::from("test1"), vec![polyline1.clone()]);

        assert_eq!(result, expected);

        //Test case 2: Extend: Start is start = True, End is start = True
        test_layers.insert(String::from("test2"), vec![start_is_start.clone(), end_is_start.clone()]);
        test_affected_layers.insert(String::from("test2"), vec![start_is_start.clone(), end_is_start.clone()]);
        let result = algorithms::try_to_close_polylines(true, &test_layers, &test_affected_layers, &Some(100.0), &Some(180), &Some(10));

    
        let x1_values = vec![1.0, 2.0, 3.0, 5.0, 6.0]; 
        let y1_values = vec![1.0, 1.0, 1.0, 3.0, 3.0];
        let polyline3 = PolyLine::new(true, x1_values, y1_values);
        expected.insert(String::from("test2"), vec![polyline3.clone()]);

        assert_eq!(result, expected);

        //Test case 3: Extend: Start is start, end is end
        test_layers.insert(String::from("test3"), vec![start_is_start.clone(), end_is_end.clone()]);
        test_affected_layers.insert(String::from("test3"), vec![start_is_start.clone(), end_is_end.clone()]);
        let result = algorithms::try_to_close_polylines(true, &test_layers, &test_affected_layers, &Some(100.0), &Some(180), &Some(10));

        
        let x1_values = vec![6.0, 5.0, 3.0, 2.0, 1.0]; 
        let y1_values = vec![3.0, 3.0, 1.0, 1.0, 1.0];
        let polyline4 = PolyLine::new(true, x1_values, y1_values);
        expected.insert(String::from("test3"), vec![polyline4.clone()]);

        assert_eq!(result, expected);

        //Test case 5: Extend: Start is end, end is start
        test_layers.insert(String::from("test4"), vec![start_is_end.clone(), end_is_start.clone()]); 
        test_affected_layers.insert(String::from("test4"), vec![start_is_end.clone(), end_is_start.clone()]);
        let result = algorithms::try_to_close_polylines(true, &test_layers, &test_affected_layers, &Some(100.0), &Some(180), &Some(10));

        let x1_values = vec![1.0, 2.0, 3.0, 5.0, 6.0]; 
        let y1_values = vec![1.0, 1.0, 1.0, 3.0, 3.0];
        let polyline5 = PolyLine::new(true, x1_values, y1_values);
        expected.insert(String::from("test4"), vec![polyline5.clone()]);

        assert_eq!(result, expected);


        //Test case 6: Extend: Start is end, end is end
        test_layers.insert(String::from("test5"), vec![start_is_end.clone(), end_is_end.clone()]); 
        test_affected_layers.insert(String::from("test5"), vec![start_is_end.clone(), end_is_end.clone()]);
        let result = algorithms::try_to_close_polylines(true, &test_layers, &test_affected_layers, &Some(100.0), &Some(180), &Some(10));

        let x1_values = vec![6.0, 5.0, 3.0, 2.0, 1.0]; 
        let y1_values = vec![3.0, 3.0, 1.0, 1.0, 1.0];
        let polyline6 = PolyLine::new(true, x1_values, y1_values);
        expected.insert(String::from("test5"), vec![polyline6.clone()]);

        assert_eq!(result, expected);

        //Test case 7: Add new layers to test_layers, BUT NOT in affected_layers => 
        //Layers not added in affected, should not be closed
        let x_values = vec![10.0, 15.0, 14.0]; 
        let y_values = vec![3.0, 3.0, 2.0];
        let not_affected1 = PolyLine::new(false, x_values, y_values);

        let x_values = vec![16.0, 15.0, 14.0]; 
        let y_values = vec![3.0, 3.0, 2.0];
        let not_affected2 = PolyLine::new(false, x_values, y_values);

        //Add two lines into to different layers
        test_layers.insert(String::from("test6"), vec![not_affected1.clone()]);
        test_layers.insert(String::from("test7"), vec![not_affected2.clone()]);

        //Add the exact same lines into expected => no changes should be made on the lines because they are not affected
        expected.insert(String::from("test6"), vec![not_affected1.clone()]);
        expected.insert(String::from("test7"), vec![not_affected2.clone()]);

        let result = algorithms::try_to_close_polylines(true, &test_layers, &test_affected_layers, &Some(100.0), &Some(180), &Some(10));

        assert_eq!(result, expected);
}

#[test]
fn test_try_to_connect_polylines(){
    //Create a Sample data set
        //BTreeMap
        let mut test_layers: BTreeMap<String, Vec<PolyLine>> = BTreeMap::new();
        let mut test_affected_layers: BTreeMap<String, Vec<PolyLine>> = BTreeMap::new();

        //Random line
        let x1_values = vec![1.0, 2.0, 2.0, 2.0, 1.0]; 
        let y1_values = vec![1.0, 1.0, 2.0, 3.0, 3.0];
        let polyline1 = PolyLine::new(false, x1_values, y1_values);

        //Start is start
        let x2_values = vec![1.0, 2.0, 3.0]; 
        let y2_values = vec![1.0, 1.0, 1.0];
        let start_is_start = PolyLine::new(false, x2_values, y2_values);

        //Start is end
        let x5_values = vec![3.0, 2.0, 1.0]; 
        let y5_values = vec![1.0, 1.0, 1.0];
        let start_is_end = PolyLine::new(false, x5_values, y5_values);

        //End is start
        let x3_values = vec![4.0, 5.0, 6.0]; 
        let y3_values = vec![2.0, 3.0, 3.0];
        let end_is_start = PolyLine::new(false, x3_values, y3_values);

        //End is end
        let x4_values = vec![6.0, 5.0, 4.0]; 
        let y4_values = vec![3.0, 3.0, 2.0];
        let end_is_end = PolyLine::new(false, x4_values, y4_values);

        let mut expected: BTreeMap<String, Vec<PolyLine>> = BTreeMap::new();

        //Test case 0: Only 1 iteration => line should give correct values, but is_closed = false
        test_layers.insert(String::from("test0"), vec![start_is_start.clone(), end_is_start.clone()]);
        test_affected_layers.insert(String::from("test0"), vec![start_is_start.clone(), end_is_start.clone()]);
        let result = algorithms::try_to_close_polylines(false, &test_layers, &test_affected_layers, &Some(100.0), &Some(180), &Some(1));

        let x1_values = vec![6.0, 5.0, 4.0, 3.0, 2.0, 1.0]; 
        let y1_values = vec![3.0, 3.0, 2.0, 1.0, 1.0, 1.0];
        let polyline3 = PolyLine::new(false, x1_values, y1_values);
        expected.insert(String::from("test0"), vec![polyline3.clone()]);

        assert_eq!(result, expected);

        
        //remove the data from this test so it do not interfere with later tests
        test_layers.remove("test0");
        test_affected_layers.remove("test0");
        expected.remove("test0");
        

        //Test case 1: Close with it's own polyline => is closed from false to true
        test_layers.insert(String::from("test1"), vec![polyline1.clone()]);
        test_affected_layers.insert(String::from("test1"), vec![polyline1.clone()]);
        let result = algorithms::try_to_close_polylines(false, &test_layers, &test_affected_layers, &Some(100.0), &Some(180), &Some(10));

        let x1_values = vec![1.0, 2.0, 2.0, 2.0, 1.0]; 
        let y1_values = vec![1.0, 1.0, 2.0, 3.0, 3.0];
        let polyline1 = PolyLine::new(true, x1_values, y1_values);
        expected.insert(String::from("test1"), vec![polyline1.clone()]);

        assert_eq!(result, expected);

        //Test case 2: Connect: Start is start, end is start
        test_layers.insert(String::from("test2"), vec![start_is_start.clone(), end_is_start.clone()]);
        test_affected_layers.insert(String::from("test2"), vec![start_is_start.clone(), end_is_start.clone()]);
        let result = algorithms::try_to_close_polylines(false, &test_layers, &test_affected_layers, &Some(100.0), &Some(180), &Some(10));

        let x1_values = vec![6.0, 5.0, 4.0, 3.0, 2.0, 1.0]; 
        let y1_values = vec![3.0, 3.0, 2.0, 1.0, 1.0, 1.0];
        let polyline3 = PolyLine::new(true, x1_values, y1_values);
        expected.insert(String::from("test2"), vec![polyline3.clone()]);

        assert_eq!(result, expected);

        //Test case 3: Connect: Start is start, end is end
        test_layers.insert(String::from("test3"), vec![start_is_start.clone(), end_is_end.clone()]);
        test_affected_layers.insert(String::from("test3"), vec![start_is_start.clone(), end_is_end.clone()]);
        let result = algorithms::try_to_close_polylines(false, &test_layers, &test_affected_layers, &Some(100.0), &Some(180), &Some(10));

        
        let x1_values = vec![6.0, 5.0, 4.0, 3.0, 2.0, 1.0]; 
        let y1_values = vec![3.0, 3.0, 2.0, 1.0, 1.0, 1.0];
        let polyline4 = PolyLine::new(true, x1_values, y1_values);
        expected.insert(String::from("test3"), vec![polyline4.clone()]);

        assert_eq!(result, expected);
        
        //Test case 4: Connect: Start is end, end is start
        test_layers.insert(String::from("test4"), vec![start_is_end.clone(), end_is_start.clone()]); 
        test_affected_layers.insert(String::from("test4"), vec![start_is_end.clone(), end_is_start.clone()]);
        let result = algorithms::try_to_close_polylines(false, &test_layers, &test_affected_layers, &Some(100.0), &Some(180), &Some(10));

        let x1_values = vec![6.0, 5.0, 4.0, 3.0, 2.0, 1.0]; 
        let y1_values = vec![3.0, 3.0, 2.0, 1.0, 1.0, 1.0];
        let polyline5 = PolyLine::new(true, x1_values, y1_values);
        expected.insert(String::from("test4"), vec![polyline5.clone()]);

        assert_eq!(result, expected);

        //Test case 5: Connect: Start is end, end is end
        test_layers.insert(String::from("test5"), vec![start_is_end.clone(), end_is_end.clone()]); 
        test_affected_layers.insert(String::from("test5"), vec![start_is_end.clone(), end_is_end.clone()]);
        let result = algorithms::try_to_close_polylines(false, &test_layers, &test_affected_layers, &Some(100.0), &Some(180), &Some(10));

        let x1_values = vec![6.0, 5.0, 4.0, 3.0, 2.0, 1.0]; 
        let y1_values = vec![3.0, 3.0, 2.0, 1.0, 1.0, 1.0];
        let polyline6 = PolyLine::new(true, x1_values, y1_values);
        expected.insert(String::from("test5"), vec![polyline6.clone()]);

        assert_eq!(result, expected);


        //Test case 6: Add new layers to test_layers, BUT NOT in affected_layers => 
        //Layers not added in affected, should not be closed
        let x_values = vec![10.0, 15.0, 14.0]; 
        let y_values = vec![3.0, 3.0, 2.0];
        let not_affected1 = PolyLine::new(false, x_values, y_values);

        let x_values = vec![16.0, 15.0, 14.0]; 
        let y_values = vec![3.0, 3.0, 2.0];
        let not_affected2 = PolyLine::new(false, x_values, y_values);

        //Add two lines into to different layers
        test_layers.insert(String::from("test6"), vec![not_affected1.clone()]);
        test_layers.insert(String::from("test7"), vec![not_affected2.clone()]);

        //Add the exact same lines into expected => no changes should be made on the lines because they are not affected
        expected.insert(String::from("test6"), vec![not_affected1.clone()]);
        expected.insert(String::from("test7"), vec![not_affected2.clone()]);

        let result = algorithms::try_to_close_polylines(false, &test_layers, &test_affected_layers, &Some(100.0), &Some(180), &Some(10));

        assert_eq!(result, expected);
}
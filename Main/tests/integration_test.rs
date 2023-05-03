#[cfg(test)]

use dxf_janitors;
use dxf_janitors::{dxfextract::PolyLine, svgwrite};
use dxf_janitors::dxfwrite;
use crate::dxf_janitors::algorithms;
use std::{collections::{HashMap, BTreeMap}, f64::consts::PI, vec, fmt, path::Path, fs::remove_file};

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

#[test]
fn test_try_save_as_dxf(){
    //Create sample data
        //BTreeMap
        let mut test_layers: BTreeMap<String, Vec<PolyLine>> = BTreeMap::new();

        //Random line
        let x1_values = vec![1.0, 2.0, 2.0, 2.0, 1.0]; 
        let y1_values = vec![1.0, 1.0, 2.0, 3.0, 3.0];
        let polyline1 = PolyLine::new(false, x1_values, y1_values);

        //Random line 2
        let x2_values = vec![1.0, 2.0, 3.0]; 
        let y2_values = vec![1.0, 1.0, 1.0];
        let polyline2 = PolyLine::new(false, x2_values, y2_values);

        //Insert lines into map
        test_layers.insert("layer1".to_string(), vec![polyline1.clone()]);
        test_layers.insert("layer2".to_string(), vec![polyline1.clone(), polyline2.clone()]);

        //Temporary output path
        let output_path = "test_save.dxf".to_string();

        //Check if save function works and that the file was created
        assert!(dxfwrite::savedxf(test_layers, &output_path).is_ok());
        assert!(Path::new(&output_path).exists());

        //remove the temp file (fs::remove_file)
        remove_file(&output_path).unwrap();
}

#[test]
    //NEED TO CHECK IF FORMATTING OF SVG DOCUMENT IS CORRECT
    fn test_create_svg(){
        let mut layer_polylines: BTreeMap<String, Vec<PolyLine>> = BTreeMap::new();

        let x1_values = vec![0.0, 1.0, 2.0]; 
        let y1_values = vec![0.0, 1.0, 0.0];
        let polyline1 = PolyLine::new(true, x1_values, y1_values);

        layer_polylines.insert("layer1".to_string(), vec![polyline1]);
        let min_x = 0.0;
        let max_y = 2.0;
        let width = 100.0;
        let height = 100.0;

        let test_doc = svgwrite::create_svg(&layer_polylines, &min_x, &max_y, &width, &height, vec![[255., 0., 0.]]);
        //rgba(65025,0,0,1.0) 65025 / 255 = 255 => rgb(255,0,0)
        assert_eq!(test_doc.to_string(), 
        "<svg inkscape:version=\"1.1.1 (3bf5ae0d25, 2021-09-20)\" viewBox=\"0 0 100 100\" xmlns=\"http://www.w3.org/2000/svg\" xmlns:inkscape=\"http://www.inkscape.org/namespaces/inkscape\">\n<g inkscape:groupmode=\"layer\" inkscape:label=\"layer1\" style=\"display:inline\">\n<path d=\"M0,2 L1,1 L2,2 z\" fill=\"none\" stroke=\"rgba(65025,0,0,1.0)\" stroke-width=\"0.1px\"/>\n</g>\n</svg>"
    );
    
        
    }

    #[test]
    fn test_calculate_min_max_(){
        //Sample data set
        let mut layer_polylines = BTreeMap::new();


        let x1_values = vec![1.0, 2.0, 3.0, 4.0];
        let y1_values = vec![1.0, 2.0, 3.0, 4.0];

        let x2_values = vec![5.0, 6.0, 7.0, 8.0];
        let y2_values = vec![5.0, 6.0, 7.0, 8.0];

        let x3_values = vec![-1.0, -2.0, -3.0, -4.0];
        let y3_values = vec![-1.0, -2.0, -3.0, -4.0];

        let polyline1 = PolyLine::new(false, x1_values, y1_values);
        let polyline2 = PolyLine::new(false, x2_values, y2_values);
        let polyline3 = PolyLine::new(false, x3_values, y3_values);

        layer_polylines.insert(String::from("layer1"), vec![polyline1, polyline2]);
        layer_polylines.insert(String::from("layer2"), vec![polyline3]);

        //Test case 1 : BTreeMap is empty
        let empty: BTreeMap<String, Vec<PolyLine>> = BTreeMap::new();
        assert!(algorithms::calculate_min_max(&empty).is_none());

        //Test case 2 : Non-Empty, use the sample data
        let expected = (-4.0, -4.0, 8.0, 12.0, 12.0); //(min_x, min_y, max_y, width, height)
        assert_eq!(algorithms::calculate_min_max(&layer_polylines).unwrap(), expected);


    }
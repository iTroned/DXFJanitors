import ezdxf
import sys
import os
import json
import matplotlib as plt
from ezdxf import recover
from ezdxf.addons.drawing import matplotlib
def savedxf(*args, **kwargs): 
    json_data = kwargs["json"]
    layers = json.loads(json_data)
    path = kwargs["path"]
    file = ezdxf.new()
    file.layers.remove("0")
    msp = file.modelspace()
    counter = 1
    for layer, polylines in layers.items():
        file.layers.add(layer)
        #print(len(polylines))
        #counter += 1
        for polyline in polylines:
            points = []
            x_values = polyline["x_values"]
            #print(x_values)
            y_values = polyline["y_values"]
            #print(y_values)

            for i in range(len(x_values)):
                points.append((x_values[i], y_values[i]))
            msp.add_lwpolyline(points)
    file.saveas(path)
import ezdxf
import sys
import os
import json
import matplotlib.pyplot as plt
from ezdxf import recover
from ezdxf.addons.drawing import matplotlib
from ezdxf.addons.drawing import RenderContext, Frontend
from ezdxf.addons.drawing.matplotlib import MatplotlibBackend
def savedxf(*args, **kwargs): 
    json_data = kwargs["json"]
    layers = json.loads(json_data)
    path = kwargs["path"]
    file = ezdxf.new("R2018", setup=True)
    file.layers.remove("0")
    msp = file.modelspace()
    counter = 1
    for layer, polylines in layers.items():
        file.layers.add(name=layer, color=counter)
        #print(len(polylines))
        counter += 1
        for polyline in polylines:
            points = []
            x_values = polyline["x_values"]
            #print(x_values)
            y_values = polyline["y_values"]
            #print(y_values)

            for i in range(len(x_values)):
                points.append((x_values[i], y_values[i]))
            #print(points)
            msp.add_lwpolyline(points, close=polyline["is_closed"], dxfattribs={"layer": layer})
    file.saveas(path)

def savesvg(*args, **kwargs):
    in_path = kwargs["in_path"]
    out_path = kwargs["out_path"]
    try:
        doc, auditor = recover.readfile(in_path)
    except IOError:
        print(f'Not a DXF file or a generic I/O error.')
        sys.exit(1)
    except ezdxf.DXFStructureError:
        print(f'Invalid or corrupted DXF file.')
        sys.exit(2)

# The auditor.errors attribute stores severe errors,
# which may raise exceptions when rendering.
    if not auditor.has_errors:
        fig = plt.figure()
        Frontend(RenderContext(doc), MatplotlibBackend(fig.add_axes([0, 0, 1, 1]))).draw_layout(doc.modelspace(), finalize=True)
        fig.savefig(out_path)
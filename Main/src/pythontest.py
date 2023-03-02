import ezdxf
import sys
import os
import matplotlib as plt
from ezdxf import recover
from ezdxf.addons.drawing import matplotlib




#take in path of dxf file to access it
#store it as doc with ezdxf readfile



def convert_svg(path):
    try:
        doc = ezdxf.readfile(path)


        msp = doc.modelspace() #every dxf file consist of one modelspace and one paperspace
        psp = doc.paperspace() 
        matplotlib.qsave(msp, 'your.svg')
    except IOError:
        print("Selected file is not a dxf")
        sys.exit(1)
    except ezdxf.DXFStructureError:
        print("Invalid or corrupted file")
        try:
            ezdxf.recover.readfile(path) #if corrupted try to recover the file before giving up
        except:
            print("Corrupted")
            sys.exit(2)
    





#doc = ezdxf.new(setup=True)

#msp = doc.modelspace()
#msp.add_line((0, 0), (1, 0), dxfattribs={"layer": "MyLayer"})

#doc.saveas("Desktop/Bachelor/new_name.dxf")




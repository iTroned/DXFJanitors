crash when loading when not selecting file first
crash when trying to save after using "extend" function
rename of svg not taking place before rebuild -> maybe make it dynamic?
cant undo after merging
trying to open a file that is open in autocad crashes the app -> permission denied error code


If the closest point is part of the same polyline:
Code line: 679
Problem: Removal of the line causes problems 
Instead of creating a connection between start and end point: It deletes point 1 of the line and connects it from point 2.
Eksempel: 
First iter: https://i.imgur.com/8T18UP3.png
Second iter: https://i.imgur.com/F3fxkhS.png

Problem: If a line is dashed odd amount of times: Ex. 3 times:
Two will connect correctly, however the last dash will think closest point is part of the same line and close automatically:
Eksempel: https://i.imgur.com/6MLSoZj.png


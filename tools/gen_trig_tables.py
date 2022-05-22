from math import ceil, floor, sin, cos, tan, pi

VARIABLES = {}

# size of tile (wall height)
VARIABLES["TILE_SIZE"] = 64
VARIABLES["WALL_HEIGHT"] = 64

# Remember that PROJECTIONPLANE = screen.  This demo assumes your screen is 320 pixels wide, 200 pixels high
VARIABLES["PROJECTION_PLANE_WIDTH"] = 320
VARIABLES["PROJECTION_PLANE_HEIGHT"] = 200

# We use FOV of 60 degrees.  So we use this FOV basis of the table, taking into account
# that we need to cast 320 rays (PROJECTION_PLANE_WIDTH) within that 60 degree FOV.
VARIABLES["ANGLE60"] = VARIABLES["PROJECTION_PLANE_WIDTH"]

# player's attributes
VARIABLES["F_PLAYER_ARC"] = VARIABLES["ANGLE60"]
VARIABLES["F_PLAYER_DIST_TO_PROJ_PLANE"] = 277

# Half of the screen height
VARIABLES["F_PROJ_PLANE_TO_CENTER"] = VARIABLES["PROJECTION_PLANE_HEIGHT"]//2

# You must make sure these values are integers because we're using loopup tables.
VARIABLES["ANGLE30"] = floor(VARIABLES["ANGLE60"]/2)
VARIABLES["ANGLE15"] = floor(VARIABLES["ANGLE30"]/2)
VARIABLES["ANGLE90"] = floor(VARIABLES["ANGLE30"]*3)
VARIABLES["ANGLE180"] = floor(VARIABLES["ANGLE90"]*2)
VARIABLES["ANGLE270"] = floor(VARIABLES["ANGLE90"]*3)
VARIABLES["ANGLE360"] = floor(VARIABLES["ANGLE60"]*6)
VARIABLES["ANGLE0"] = 0
VARIABLES["ANGLE5"] = floor(VARIABLES["ANGLE30"]/6)
VARIABLES["ANGLE3"] = floor(VARIABLES["ANGLE30"]/10)
VARIABLES["ANGLE10"] = floor(VARIABLES["ANGLE5"]*2)
VARIABLES["ANGLE45"] = floor(VARIABLES["ANGLE15"]*3)

# trigonometric tables (the ones with "I" such as ISiTable are "Inverse" table)
VARIABLES["F_SIN_TABLE"]=[]
VARIABLES["F_I_SIN_TABLE"]=[]
VARIABLES["F_COS_TABLE"]=[]
VARIABLES["F_I_COS_TABLE"]=[]
VARIABLES["F_TAN_TABLE"]=[]
VARIABLES["F_I_TAN_TABLE"]=[]
VARIABLES["F_FISH_TABLE"]=[]
VARIABLES["F_X_STEP_TABLE"]=[]
VARIABLES["F_Y_STEP_TABLE"]=[]


def populate_tables():
    i = 0
    radian = 0
    VARIABLES["F_SIN_TABLE"] = [0] * (VARIABLES["ANGLE360"]+1)
    VARIABLES["F_I_SIN_TABLE"] = [0] * (VARIABLES["ANGLE360"]+1)
    VARIABLES["F_COS_TABLE"] = [0] * (VARIABLES["ANGLE360"]+1)
    VARIABLES["F_I_COS_TABLE"] = [0] * (VARIABLES["ANGLE360"]+1)
    VARIABLES["F_TAN_TABLE"] = [0] * (VARIABLES["ANGLE360"]+1)
    VARIABLES["F_I_TAN_TABLE"] = [0] * (VARIABLES["ANGLE360"]+1)
    VARIABLES["F_FISH_TABLE"] = [0] * (VARIABLES["ANGLE360"]+1)
    VARIABLES["F_X_STEP_TABLE"] = [0] * (VARIABLES["ANGLE360"]+1)
    VARIABLES["F_Y_STEP_TABLE"] = [0] * (VARIABLES["ANGLE360"]+1)

    for i in range(0, VARIABLES["ANGLE360"]+1):
        # Populate tables with their radian values.
        # (The addition of 0.0001 is a kludge to avoid divisions by 0. Removing it will produce unwanted holes in the wall when a ray is at 0, 90, 180, or 270 degree angles)
        radian = arcToRad(i) + (0.0001)
        VARIABLES["F_SIN_TABLE"][i]=sin(radian)
        VARIABLES["F_I_SIN_TABLE"][i]=(1.0/(VARIABLES["F_SIN_TABLE"][i]))
        VARIABLES["F_COS_TABLE"][i]=cos(radian)
        VARIABLES["F_I_COS_TABLE"][i]=(1.0/(VARIABLES["F_COS_TABLE"][i]))
        VARIABLES["F_TAN_TABLE"][i]=tan(radian)
        VARIABLES["F_I_TAN_TABLE"][i]=(1.0/VARIABLES["F_TAN_TABLE"][i])

        # Next we crate a table to speed up wall lookups.
        # 
        #  You can see that the distance between walls are the same
        #  if we know the angle
        #  _____|_/next xi______________
        #       |
        #  ____/|next xi_________   slope = tan = height / dist between xi's
        #     / |
        #  __/__|_________  dist between xi = height/tan where height=tile size
        # old xi|
        #                  distance between xi = x_step[view_angle]
        
        
        
        # Facing LEFT
        if (i>=VARIABLES["ANGLE90"] and i<VARIABLES["ANGLE270"] ):
            VARIABLES["F_X_STEP_TABLE"][i] = (VARIABLES["TILE_SIZE"] /VARIABLES["F_TAN_TABLE"][i])
            if (VARIABLES["F_X_STEP_TABLE"][i]>0):
                VARIABLES["F_X_STEP_TABLE"][i]=-VARIABLES["F_X_STEP_TABLE"][i]
        # facing RIGHT
        else:
            VARIABLES["F_X_STEP_TABLE"][i] = (VARIABLES["TILE_SIZE"] /VARIABLES["F_TAN_TABLE"][i])
            if (VARIABLES["F_X_STEP_TABLE"][i]<0):
                VARIABLES["F_X_STEP_TABLE"][i]=-VARIABLES["F_X_STEP_TABLE"][i]

        # FACING DOWN
        if (i>=VARIABLES["ANGLE0"]  and i<VARIABLES["ANGLE180"] ):
            VARIABLES["F_Y_STEP_TABLE"][i] = (VARIABLES["TILE_SIZE"] *VARIABLES["F_TAN_TABLE"][i])
            if (VARIABLES["F_Y_STEP_TABLE"][i]<0):
                VARIABLES["F_Y_STEP_TABLE"][i]=-VARIABLES["F_Y_STEP_TABLE"][i]
        # FACING UP
        else:
            VARIABLES["F_Y_STEP_TABLE"][i] = (VARIABLES["TILE_SIZE"] *VARIABLES["F_TAN_TABLE"][i])
            if (VARIABLES["F_Y_STEP_TABLE"][i]>0):
                VARIABLES["F_Y_STEP_TABLE"][i]=-VARIABLES["F_Y_STEP_TABLE"][i]
    # Create table for fixing FISHBOWL distortion
    for i in range(-VARIABLES["ANGLE30"], VARIABLES["ANGLE30"]+1):
        radian = arcToRad(i)
        # we don't have negative angle, so make it start at 0
        # this will give range from column 0 to 319 (PROJECTONPLANEWIDTH) since we only will need to use those range
        VARIABLES["F_FISH_TABLE"][i+VARIABLES["ANGLE30"] ] = (1.0/cos(radian))

def arcToRad(arcAngle):
    return ((arcAngle*pi)/VARIABLES["ANGLE180"]);    

def print_rust_constants():
    """
    I had to do this, and I thought hardcoding the values was
    better than calculating them on each run. I'm having second
    thoughts, but meh... I don't really care that much.
    """
    for var_name, var_value in VARIABLES.items():
        if isinstance(var_value, int):
            print(f"pub const {var_name}:u16 = {var_value};")
        elif isinstance(var_value, list):
            values = ",".join([format(float(value), ".17") for value in var_value]) # force 0 to behave
            print(f"pub const {var_name}:[f32;{VARIABLES['ANGLE360']+1}] = [{values}];")
        else:
            raise Exception(f"Unknown type: {var_value.__class__}. {var_name}={var_value}")

if __name__=='__main__':
    populate_tables()
    print("//generated code. do not edit by hand!")
    print("//python3 tools/gen_trig_tables.py > web_app/src/my_game/precalc.rs")
    print_rust_constants()


#version 450
layout(location=0)out vec4 o_Target;
layout(location=2)in vec2 v_Uv;
layout(set=2,binding=0)uniform ColorMaterial_color{
  vec4 color;
};

#define PI 3.14159265359
#define TWO_PI 6.28318530718

void main(){
  vec2 st=v_Uv.xy;
  vec3 c=vec3(0.);
  float d=0.;
  
  // Remap the space to -1. to 1.
  st=st*2.-1.;
  
  // Number of sides of your shape
  int N=3;
  
  // Angle and radius from the current pixel
  float a=atan(st.x,-st.y)+PI;
  float r=TWO_PI/float(N);
  
  // Shaping function that modulate the distance
  d=cos(floor(.5+a/r)*r-a)*length(st);
  
  d=1.-smoothstep(.4,.41,d);
  if(d<.1){
    discard;
  }
  c=vec3(d);
  o_Target=vec4(c,1.)*color;
}
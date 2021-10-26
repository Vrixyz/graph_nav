#version 300 es

precision mediump float;

out vec4 o_Target;

uniform vec2 u_resolution;
uniform vec2 u_mouse;
uniform float u_time;

struct ColorMaterial_color
{
  vec4 color;
};
uniform ColorMaterial_color mat;

void main(){
  vec2 st=gl_FragCoord.xy/u_resolution.xy;
  st.x*=u_resolution.x/u_resolution.y;
  
  vec3 color=vec3(0.);
  color=vec3(st.x,st.y,abs(sin(u_time)));
  
  o_Target=vec4(color,1.)*(mat.color+.9);
}
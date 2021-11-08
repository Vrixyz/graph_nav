#version 300 es
precision lowp float;

// TODO: this code works on https://www.shadertoy.com/new : adapt it to bevy: start from an empty shader and use hot reloading to succeed
#define PI 3.14
#define TWO_PI 6.28

out vec4 o_Target;
in vec2 v_Uv;

uniform CircleGaugeMaterial_color
{
  vec4 color;
};

// TODO: Problematic code...
uniform CircleGaugeMaterial_ratio
{
  float ratio;
};

float hardCircle(in vec2 uv,in float radius,in float width)
{
  return smoothstep(width,width*.99,abs(radius-length(uv)));
}

float cutSector(in vec2 uv,in float cutAngle,in float offset)
{
  float angle=atan(uv.y,-uv.x)+PI+offset;
  angle=mod(angle,TWO_PI);
  return smoothstep(cutAngle,cutAngle-.0001,abs(angle-cutAngle));
}

void main(){
  // Normalized pixel coordinates (from 0 to 1)
  vec2 uv=v_Uv-vec2(.5,.5);
  float c=hardCircle(uv,.3,.01)+hardCircle(uv,.4,.04)*(cutSector(uv,TWO_PI*.5*abs(ratio),TWO_PI*.25));
  
  if(c<=0.){
    discard;
  }
  vec3 col=vec3(c);
  // Output to screen
  o_Target=vec4(col,1.)*color;
}
#version 450
layout(location=0)out vec4 o_Target;
layout(location=2)in vec2 v_Uv;

float draw_circle(vec2 coord,float radius){
    float pct=length(coord-vec2(.5));
    pct=1.-pct;
    pct=smoothstep(.5,.6,pct);
    return pct;
}
float draw_circle_hard(vec2 coord,float radius){
    return step(length(coord),radius);
}
void main(){
    float circle=draw_circle_hard(v_Uv-vec2(.5),.5);
    
    if(circle<.01){
        discard;
    }
    vec3 color=vec3(circle*.6f,0.,0.);
    o_Target=vec4(color,circle);
}
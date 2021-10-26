// TODO: this code works on https://www.shadertoy.com/new : adapt it to bevy: start from an empty shader and use hot reloading to succeed

#define PI 3.14
#define TWO_PI 6.28

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

void mainImage(out vec4 fragColor,in vec2 fragCoord)
{
    float time=iTime;
    
    // Normalized pixel coordinates (from 0 to 1)
    vec2 uv=fragCoord/iResolution.xy-vec2(.5,.5);
    float c=hardCircle(uv,.4,.05)*(cutSector(uv,TWO_PI*.5*abs(sin(time)),TWO_PI*.25));
    
    vec3 col=vec3(c);
    
    // Output to screen
    fragColor=vec4(col,1.);
}
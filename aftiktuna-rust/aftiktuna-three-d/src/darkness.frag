uniform vec2 center;
uniform float radius;
uniform float intensity;

in vec4 gl_FragCoord;

layout (location = 0) out vec4 outColor;

void main()
{
    float dist = distance(center, gl_FragCoord.xy);
    float mod = min(1, dist / radius);
    outColor = vec4(0, 0, 0, intensity * mod);
}

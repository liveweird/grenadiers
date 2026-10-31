export const finishVertexShader = `
varying vec2 vUv;
void main() {
  vUv = uv;
  gl_Position = vec4(position.xy, 0.0, 1.0);
}`

export const finishFragmentShader = `
uniform sampler2D sourceMap;
varying vec2 vUv;
float bayer4(vec2 pixel) {
  vec2 p = mod(floor(pixel), 4.0);
  float x = p.x;
  float y = p.y;
  if (y < 1.0) return (x < 1.0 ? 0.0 : x < 2.0 ? 8.0 : x < 3.0 ? 2.0 : 10.0) / 16.0;
  if (y < 2.0) return (x < 1.0 ? 12.0 : x < 2.0 ? 4.0 : x < 3.0 ? 14.0 : 6.0) / 16.0;
  if (y < 3.0) return (x < 1.0 ? 3.0 : x < 2.0 ? 11.0 : x < 3.0 ? 1.0 : 9.0) / 16.0;
  return (x < 1.0 ? 15.0 : x < 2.0 ? 7.0 : x < 3.0 ? 13.0 : 5.0) / 16.0;
}
void main() {
  vec3 color = texture2D(sourceMap, vUv).rgb;
  float luminance = dot(color, vec3(0.2126, 0.7152, 0.0722));
  color = mix(vec3(luminance), color, 1.045) * vec3(1.025, 1.0, 0.965);
  color = pow(max(color, 0.0), vec3(0.97));
  color += (bayer4(gl_FragCoord.xy) - 0.5) / 255.0;
  gl_FragColor = vec4(color, 1.0);
  #include <colorspace_fragment>
}`

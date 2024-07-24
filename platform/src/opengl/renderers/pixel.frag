#version 330 core

#ifdef GL_ES
    #ifdef GL_FRAGMENT_PRECISION_HIGH
        precision highp float;
    #else
        precision mediump float;
    #endif
#endif

uniform sampler2D u_texture;

in vec2 v_texCoords;

out vec4 FragColor;

void main() {
    FragColor = texture(u_texture, v_texCoords);
}
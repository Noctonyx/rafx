(m      #     �                GLSL.std.450                     main    �   \  5  v  x  z  �               �   	 GL_ARB_separate_shader_objects   	 GL_ARB_shading_language_420pack  
 GL_GOOGLE_cpp_style_line_directive    GL_GOOGLE_include_directive      main         normal_map(mf33;vf2;         tangent_binormal_normal      uv   
    calculate_percent_lit(vf3;i1;        normal       index        spotlight_cone_falloff(vf3;vf3;f1;       surface_to_light_dir         spotlight_dir        spotlight_half_angle      $   ndf_ggx(vf3;vf3;f1;   !   n     "   h     #   roughness     )   geometric_attenuation_schlick_ggx(f1;f1;      '   dot_product   (   k     0   geometric_attenuation_smith(vf3;vf3;vf3;f1;   ,   n     -   v     .   l     /   roughness    
 6   fresnel_schlick(vf3;vf3;vf3;      3   v     4   h     5   fresnel_base      A   shade_pbr(vf3;vf3;vf3;vf3;vf3;f1;f1;vf3;      9   surface_to_light_dir_vs   :   surface_to_eye_dir_vs     ;   normal_vs     <   F0    =   base_color    >   roughness     ?   metalness     @   radiance      C   PointLight    C       position_ws   C      position_vs   C      color     C      range     C      intensity     C      shadow_map    N   point_light_pbr(struct-PointLight-vf3-vf3-vf4-f1-f1-i11;vf3;vf3;vf3;vf3;vf3;f1;f1;    F   light     G   surface_to_eye_dir_vs     H   surface_position_vs   I   normal_vs     J   F0    K   base_color    L   roughness     M   metalness     P   SpotLight     P       position_ws   P      direction_ws      P      position_vs   P      direction_vs      P      color    	 P      spotlight_half_angle      P      range     P      intensity     P      shadow_map    [   spot_light_pbr(struct-SpotLight-vf3-vf3-vf3-vf3-vf4-f1-f1-f1-i11;vf3;vf3;vf3;vf3;vf3;f1;f1;   S   light     T   surface_to_eye_dir_vs     U   surface_position_vs   V   normal_vs     W   F0    X   base_color    Y   roughness     Z   metalness     ]   DirectionalLight      ]       direction_ws      ]      direction_vs      ]      color     ]      intensity     ]      shadow_map    h   directional_light_pbr(struct-DirectionalLight-vf3-vf3-vf4-f1-i11;vf3;vf3;vf3;vf3;vf3;f1;f1;   `   light     a   surface_to_eye_dir_vs     b   surface_position_vs   c   normal_vs     d   F0    e   base_color    f   roughness     g   metalness    
 r   pbr_path(vf3;vf4;vf4;f1;f1;vf3;   l   surface_to_eye_vs     m   base_color    n   emissive_color    o   metalness     p   roughness     q   normal_vs     t   normal    w   normal_texture    {   smp   �   shadow_map_pos    �   PointLight    �       position_ws   �      position_vs   �      color     �      range     �      intensity     �      shadow_map    �   DirectionalLight      �       direction_ws      �      direction_vs      �      color     �      intensity     �      shadow_map    �   SpotLight     �       position_ws   �      direction_ws      �      position_vs   �      direction_vs      �      color    	 �      spotlight_half_angle      �      range     �      intensity     �      shadow_map    �   ShadowMapData    	 �       shadow_map_view_proj     	 �      shadow_map_light_dir      �   PerViewData   �       ambient_light     �      point_light_count    	 �      directional_light_count   �      spot_light_count      �      point_lights      �      directional_lights    �      spot_lights   �      shadow_map_count      �      shadow_maps   �   per_view_data     �   in_position_ws    �   light_dir     �   PerObjectData     �       model     �      model_view    �      model_view_proj   �   per_object_data   �   projected     �   distance_from_light   �   sample_location   �   surface_to_light_dir      �   bias      �   shadow    �   shadow_map_images     �   smp_depth     �   cos_angle     �   min_cos   �   max_cos     a       a2    	  n_dot_h     bottom_part     bottom    !  bottom    -  r_plus_1      0  k     6  v_factor      ;  param     <  param     ?  l_factor      D  param     E  param     M  v_dot_h   b  halfway_dir_vs    g  NDF   h  param     j  param     l  param     o  G     p  param     r  param     t  param     v  param     y  F     z  param     |  param     ~  param     �  fresnel_specular      �  fresnel_diffuse   �  n_dot_l   �  n_dot_v   �  top   �  bottom    �  specular      �  surface_to_light_dir_vs   �  distance      �  attenuation   �  radiance      �  param     �  param     �  param     �  param     �  param     �  param     �  param     �  param     �  surface_to_light_dir_vs   �  distance      �  attenuation  
 �  spotlight_direction_intensity     �  param     �  param     �  param       radiance        param       param       param       param       param       param       param       param        surface_to_light_dir_vs   $  attenuation   %  radiance      .  param     0  param     2  param     4  param     6  param     8  param     :  param     <  param     A  fresnel_base      J  total_light   M  i     \  in_position_vs    ]  param     m  param     o  param     q  param     s  param     u  param     x  param     z  param     �  i     �  percent_lit   �  param     �  param     �  param     �  param     �  param     �  param     �  param     �  param     �  param     �  param     �  i     �  percent_lit   �  param     �  param     �  param     �  param     �  param     �  param     �  param     �  param       param       param       ambient     color     $  base_color    %  MaterialData      %      base_color_factor     %     emissive_factor   %     metallic_factor   %     roughness_factor     	 %     normal_texture_scale     
 %     occlusion_texture_strength    %     alpha_cutoff     	 %     has_base_color_texture    %     has_metallic_roughness_texture    %  	   has_normal_texture   	 %  
   has_occlusion_texture    	 %     has_emissive_texture      &  MaterialDataUbo   &      data      (  per_material_data     0  base_color_texture    5  in_uv     :  emissive_color    G  emissive_texture      P  metalness     T  roughness     \  sampled  	 ]  metallic_roughness_texture    u  tbn   v  in_tangent_vs     x  in_binormal_vs    z  in_normal_vs      �  normal_vs     �  param     �  param     �  eye_position_vs   �  surface_to_eye_vs     �  out_color     �  param     �  param     �  param     �  param     �  param     �  param     �  occlusion_texture   G  w   "      G  w   !      G  {   "       G  {   !      H  �       #       H  �      #      H  �      #       H  �      #   0   H  �      #   4   H  �      #   8   G  �      @   H  �       #       H  �      #      H  �      #       H  �      #   0   H  �      #   4   G  �      @   H  �       #       H  �      #      H  �      #       H  �      #   0   H  �      #   @   H  �      #   P   H  �      #   T   H  �      #   X   H  �      #   \   G  �      `   H  �          H  �       #       H  �             H  �      #   @   G  �      P   H  �       #       H  �      #      H  �      #      H  �      #      H  �      #       H  �      #      H  �      #      H  �      #      H  �      #   0  G  �      G  �   "       G  �   !       G  �         H  �          H  �       #       H  �             H  �         H  �      #   @   H  �            H  �         H  �      #   �   H  �            G  �      G  �   "      G  �   !       G  �   "       G  �   !      G  �   "       G  �   !      G  \         H  %      #       H  %     #      H  %     #      H  %     #       H  %     #   $   H  %     #   (   H  %     #   ,   H  %     #   0   H  %     #   4   H  %  	   #   8   H  %  
   #   <   H  %     #   @   H  &      #       G  &     G  (  "      G  (  !       G  0  "      G  0  !      G  5        G  G  "      G  G  !      G  ]  "      G  ]  !      G  v        G  x        G  z        G  �         G  �  "      G  �  !           !                                          	           
                  
              !        	                                          !                          !                 !  &            !  +                  !  2               !  8                                C                        D      C   !  E      D                          P                                 Q      P   !  R      Q                          ]                     ^      ]   !  _      ^                           j         ! 	 k         j   j             	 u                               v       u   ;  v   w         y      z       y   ;  z   {         }   u   +     �      @+     �     �?+     �         �             �                     +  �   �        �   �   �     �                    �   �   �     �                                �   �   �     �           �   �      +  �   �   0     �   �   �     �      �   �   �   �   �   �   �   �      �      �   ;  �   �      +     �      +     �          �      �      �         ;  �   �        �   �   �   �      �      �   ;  �   �      +     �         �         +  �   �      +  �   �      +     �      ?  �   u   �      �       �   ;  �   �       ;  z   �        	 �                             �   �   +       �I@+     4     A+     V  Y���+     Y  v�@,     �  �   �   �   +     �    �@+     �  o�:+     �     +     �     +     �     +     �     +          +     B  
�#=,     C  B  B  B  ,     K  �   �   �      L     �   +  �   N         U     �     X     [        ;  [  \        ^     �   +     �        �        +     �  ����   �     �      �     �                %                       �   �   �   �   �     &  %     '     &  ;  '  (     ;  v   0         4     
   ;  4  5     +     A     ;  v   G      ,     O  �   �   �   �      Q        ;  v   ]      +  �   g     +     o  	   ;  [  v     ;  [  x     ;  [  z        �        ;  �  �     ;  v   �      6               �     ;  j   $     ;  j   :     ;     P     ;     T     ;  j   \     ;  	   u     ;     �     ;  	   �     ;     �     ;     �     ;     �     ;     �     ;  j   �     ;  j   �     ;     �     ;     �     ;     �     A    )  (  �   �   =     *  )  >  $  *  A  U  +  (  �     =  �   ,  +  �  X  -  ,  N  �  /      �  -  .  /  �  .  =  u   1  0  =  y   2  {   V  }   3  1  2  =  
   6  5  W     7  3  6  =     8  $  �     9  8  7  >  $  9  �  /  �  /  A  �   ;  (  �   �   =     <  ;  Q     =  <      Q     >  <     Q     ?  <     P     @  =  >  ?  �   >  :  @  A  U  B  (  �   A  =  �   C  B  �  X  D  C  N  �  F      �  D  E  F  �  E  =  u   H  G  =  y   I  {   V  }   J  H  I  =  
   K  5  W     L  J  K  =     M  :  �     N  M  L  >  :  N  >  $  O  �  F  �  F  A  Q  R  (  �   �  =     S  R  >  P  S  A  Q  U  (  �   �  =     V  U  >  T  V  A  U  W  (  �   �   =  �   X  W  �  X  Y  X  N  �  [      �  Y  Z  [  �  Z  =  u   ^  ]  =  y   _  {   V  }   `  ^  _  =  
   a  5  W     b  `  a  >  \  b  A     c  \  N  =     d  c  =     e  P  �     f  e  d  >  P  f  A     h  \  g  =     i  h  =     j  T  �     k  j  i  >  T  k  �  [  �  [  =     l  T  �     m  l  �   �     n  m  �   >  T  n  A  U  p  (  �   o  =  �   q  p  �  X  r  q  N  �  t      �  r  s  �  �  s  =     w  v  =     y  x  =     {  z  Q     |  w      Q     }  w     Q     ~  w     Q       y      Q     �  y     Q     �  y     Q     �  {      Q     �  {     Q     �  {     P     �  |  }  ~  P     �    �  �  P     �  �  �  �  P     �  �  �  �  >  u  �  =     �  u  >  �  �  =  
   �  5  >  �  �  9     �     �  �  O     �  �  �            >  �  �  �  t  �  �  =     �  z  Q     �  �      Q     �  �     Q     �  �     P     �  �  �  �  �        �     E   �  O     �  �  �            >  �  �  �  t  �  t  >  �  K  =     �  �  =     �  \  �     �  �  �       �     E   �  >  �  �  =     �  �  >  �  �  =     �  $  >  �  �  =     �  :  >  �  �  =     �  P  >  �  �  =     �  T  >  �  �  =     �  �  >  �  �  9 
    �  r   �  �  �  �  �  �  >  �  �  �  8  6               7  	      7        �     ;     t      =  u   x   w   =  y   |   {   V  }   ~   x   |   =  
         W     �   ~      O     �   �   �             >  t   �   =     �   t   �     �   �   �   P     �   �   �   �   �     �   �   �   >  t   �   =     �      =     �   t   �     �   �   �   >  t   �   =     �   t   Q     �   �       Q     �   �      Q     �   �      P     �   �   �   �   �        �      E   �   �  �   8  6               7        7        �     ;  j   �      ;     �      ;     �      ;     �      ;     �      ;     �      ;     �      ;     �      =     �      A  �   �   �   �   �   �   =  �   �   �   =     �   �   �     �   �   �   >  �   �   A  �   �   �   �   =  �   �   �   Q     �   �       O     �   �   �             Q     �   �      O     �   �   �             Q     �   �      O     �   �   �             P     �   �   �   �   =     �      A  �   �   �   �   �   �   =     �   �   �     �   �   �   >  �   �   =     �   �   O     �   �   �             A     �   �   �   =     �   �   P     �   �   �   �   �     �   �   �   >  �   �   A     �   �   �   =     �   �   >  �   �   =     �   �   O  
   �   �   �          �  
   �   �   �   P  
   �   �   �   �  
   �   �   �   >  �   �   =     �   �        �   �   >  �   �   >  �   �   =     �      A  v   �   �   �   =  u   �   �   =  y   �   �   V  �   �   �   �   =  
   �   �   =     �   �   =     �   �   �     �   �   �   Q     �   �       Q     �   �      P     �   �   �   �   Q     �   �      Y     �   �   �   �   >  �   �   =     �   �   �  �   8  6               7        7        7        �      ;     �      ;     �      ;     �      =     �           �   �   =     �      �     �   �   �   >  �   �   =     �           �         �   >  �   �   =     �   �        �      .   �   �   �   >  �   �   =     �   �   =     �   �   =     �   �        �      1   �   �   �   �  �   8  6     $          7     !   7     "   7     #   �  %   ;          ;          ;     	     ;          ;          =       #   =       #   �           >      =         =         �           >      =     
  !   =       "   �       
              (     �   >  	    =       	  =       	  �           =         �         �   �           �         �   >      =         �           =         �           >      =         =         �           �    8  6     )       &   7     '   7     (   �  *   ;     !     =     "  '   =     #  (   �     $  �   #  �     %  "  $  =     &  (   �     '  %  &  >  !  '  =     (  '   =     )  !  �     *  (  )  �  *  8  6     0       +   7     ,   7     -   7     .   7     /   �  1   ;     -     ;     0     ;     6     ;     ;     ;     <     ;     ?     ;     D     ;     E     =     .  /   �     /  .  �   >  -  /  =     1  -  =     2  -  �     3  1  2  �     5  3  4  >  0  5  =     7  ,   =     8  -   �     9  7  8       :     (   9  �   >  ;  :  =     =  0  >  <  =  9     >  )   ;  <  >  6  >  =     @  ,   =     A  .   �     B  @  A       C     (   B  �   >  D  C  =     F  0  >  E  F  9     G  )   D  E  >  ?  G  =     H  6  =     I  ?  �     J  H  I  �  J  8  6     6       2   7     3   7     4   7     5   �  7   ;     M     =     N  3   =     O  4   �     P  N  O       Q     (   P  �   >  M  Q  =     R  5   =     S  5   P     T  �   �   �   �     U  T  S  =     W  M  �     X  V  W  �     Z  X  Y  =     [  M  �     \  Z  [       ]        \  �     ^  U  ]  �     _  R  ^  �  _  8  6     A       8   7     9   7     :   7     ;   7     <   7     =   7     >   7     ?   7     @   �  B   ;     b     ;     g     ;     h     ;     j     ;     l     ;     o     ;     p     ;     r     ;     t     ;     v     ;     y     ;     z     ;     |     ;     ~     ;     �     ;     �     ;     �     ;     �     ;     �     ;     �     ;     �     =     c  9   =     d  :   �     e  c  d       f     E   e  >  b  f  =     i  ;   >  h  i  =     k  b  >  j  k  =     m  >   >  l  m  9     n  $   h  j  l  >  g  n  =     q  ;   >  p  q  =     s  :   >  r  s  =     u  9   >  t  u  =     w  >   >  v  w  9     x  0   p  r  t  v  >  o  x  =     {  :   >  z  {  =     }  b  >  |  }  =       <   >  ~    9     �  6   z  |  ~  >  y  �  =     �  y  >  �  �  =     �  �  �     �  �  �  >  �  �  =     �  ?   �     �  �   �  =     �  �  �     �  �  �  >  �  �  =     �  ;   =     �  9   �     �  �  �       �     (   �  �   >  �  �  =     �  ;   =     �  :   �     �  �  �       �     (   �  �   >  �  �  =     �  g  =     �  o  �     �  �  �  =     �  y  �     �  �  �  >  �  �  =     �  �  �     �  �  �  =     �  �  �     �  �  �  >  �  �  =     �  �  =     �  �       �     (   �  �  P     �  �  �  �  �     �  �  �  >  �  �  =     �  �  =     �  =   �     �  �  �  P     �        �     �  �  �  =     �  �  �     �  �  �  =     �  @   �     �  �  �  =     �  �  �     �  �  �  �  �  8  6     N       E   7  D   F   7     G   7     H   7     I   7     J   7     K   7     L   7     M   �  O   ;     �     ;     �     ;     �     ;     �     ;     �     ;     �     ;     �     ;     �     ;     �     ;     �     ;     �     ;     �     A     �  F   �   =     �  �  =     �  H   �     �  �  �  >  �  �  =     �  �       �     B   �  >  �  �  =     �  �  =     �  �  P     �  �  �  �  �     �  �  �  >  �  �  =     �  �  =     �  �  �     �  �  �  �     �  �   �  >  �  �  A  j   �  F   �  =     �  �  O     �  �  �            =     �  �  �     �  �  �  A     �  F   �  =     �  �  �     �  �  �  >  �  �  =     �  �  >  �  �  =     �  G   >  �  �  =     �  I   >  �  �  =     �  J   >  �  �  =     �  K   >  �  �  =     �  L   >  �  �  =     �  M   >  �  �  =     �  �  >  �  �  9     �  A   �  �  �  �  �  �  �  �  �  �  8  6     [       R   7  Q   S   7     T   7     U   7     V   7     W   7     X   7     Y   7     Z   �  \   ;     �     ;     �     ;     �     ;     �     ;     �     ;     �     ;     �     ;          ;          ;          ;          ;          ;          ;          ;          ;          A     �  S   �  =     �  �  =     �  U   �     �  �  �  >  �  �  =     �  �       �     B   �  >  �  �  =     �  �  =     �  �  P     �  �  �  �  �     �  �  �  >  �  �  =     �  �  =     �  �  �     �  �  �  �     �  �   �  >  �  �  =     �  �  >  �  �  A     �  S   �  =     �  �  >  �  �  A     �  S   �  =     �  �  >  �  �  9           �  �  �  >  �     A  j     S   �  =         O                     =       �  �           A       S     =     	    �     
    	  =       �  �       
    >      =       �  >      =       T   >      =       V   >      =       W   >      =       X   >      =       Y   >      =       Z   >      =         >      9       A                   �    8  6     h       _   7  ^   `   7     a   7     b   7     c   7     d   7     e   7     f   7     g   �  i   ;           ;     $     ;     %     ;     .     ;     0     ;     2     ;     4     ;     6     ;     8     ;     :     ;     <     A     !  `   �   =     "  !       #  "  >     #  >  $  �   A  j   &  `   �  =     '  &  O     (  '  '            =     )  $  �     *  (  )  A     +  `   �  =     ,  +  �     -  *  ,  >  %  -  =     /     >  .  /  =     1  a   >  0  1  =     3  c   >  2  3  =     5  d   >  4  5  =     7  e   >  6  7  =     9  f   >  8  9  =     ;  g   >  :  ;  =     =  %  >  <  =  9     >  A   .  0  2  4  6  8  :  <  �  >  8  6     r       k   7     l   7  j   m   7  j   n   7     o   7     p   7     q   �  s   ;     A     ;     J     ;  L  M     ;  D   ]     ;     m     ;     o     ;     q     ;     s     ;     u     ;     x     ;     z     ;  L  �     ;     �     ;     �     ;     �     ;     �     ;  Q   �     ;     �     ;     �     ;     �     ;     �     ;     �     ;     �     ;     �     ;  L  �     ;     �     ;     �     ;     �     ;     �     ;  ^   �     ;     �     ;     �     ;     �     ;     �     ;     �     ;          ;          ;          ;          >  A  C  =     D  A  =     E  m   O     F  E  E            =     G  o   P     H  G  G  G       I     .   D  F  H  >  A  I  >  J  K  >  M  N  �  O  �  O  �  Q  R      �  S  �  S  =  �   T  M  A  U  V  �   �   =  �   W  V  �  X  Y  T  W  �  Y  P  Q  �  P  =  �   Z  M  A  ^  _  �   �  Z  =  �   `  _  Q     a  `      A     b  ]  �   >  b  a  Q     c  `     A     d  ]  �   >  d  c  Q     e  `     A  j   f  ]  �  >  f  e  Q     g  `     A     h  ]  �  >  h  g  Q     i  `     A     j  ]  �  >  j  i  Q     k  `     A     l  ]  �  >  l  k  =     n  l   >  m  n  =     p  \  >  o  p  =     r  q   >  q  r  =     t  A  >  s  t  =     v  m   O     w  v  v            >  u  w  =     y  p   >  x  y  =     {  o   >  z  {  9     |  N   ]  m  o  q  s  u  x  z  =     }  J  �     ~  }  |  >  J  ~  �  R  �  R  =  �     M  �  �   �    �   >  M  �  �  O  �  Q  >  �  N  �  �  �  �  �  �  �      �  �  �  �  =  �   �  �  A  U  �  �   �  =  �   �  �  �  X  �  �  �  �  �  �  �  �  �  =  �   �  �  A  �  �  �   �  �  �   =     �  �  �  X  �  �  �  �  �      �  �  �  �  �  �  =  �   �  �  =     �  q   >  �  �  A  �  �  �   �  �  �   =     �  �  >  �  �  9     �     �  �  >  �  �  �  �  �  �  >  �  �   �  �  �  �  =     �  �  >  �  �  =     �  �  =  �   �  �  A  �  �  �   �  �  =  �   �  �  Q     �  �      A     �  �  �   >  �  �  Q     �  �     A     �  �  �   >  �  �  Q     �  �     A     �  �  �  >  �  �  Q     �  �     A     �  �  �  >  �  �  Q     �  �     A  j   �  �  �  >  �  �  Q     �  �     A     �  �  �  >  �  �  Q     �  �     A     �  �  �  >  �  �  Q     �  �     A     �  �    >  �  �  Q     �  �     A     �  �  �   >  �  �  =     �  l   >  �  �  =     �  \  >  �  �  =     �  q   >  �  �  =     �  A  >  �  �  =     �  m   O     �  �  �            >  �  �  =     �  p   >  �  �  =     �  o   >  �  �  9     �  [   �  �  �  �  �  �  �  �  �     �  �  �  =     �  J  �     �  �  �  >  J  �  �  �  �  �  =  �   �  �  �  �   �  �  �   >  �  �  �  �  �  �  >  �  N  �  �  �  �  �  �  �      �  �  �  �  =  �   �  �  A  U  �  �   �  =  �   �  �  �  X  �  �  �  �  �  �  �  �  �  =  �   �  �  A  �  �  �   �  �  �  =     �  �  �  X  �  �  �  �  �      �  �  �  �  �  �  =  �   �  �  =     �  q   >  �  �  A  �  �  �   �  �  �  =     �  �  >  �  �  9     �     �  �  >  �  �  �  �  �  �  >  �  �   �  �  �  �  =     �  �  >  �  �  =     �  �  =  �   �  �  A  �  �  �   �  �  =  �   �  �  Q     �  �      A     �  �  �   >  �  �  Q     �  �     A     �  �  �   >  �  �  Q     �  �     A  j   �  �  �  >  �  �  Q     �  �     A     �  �  �  >  �  �  Q     �  �     A     �  �  �  >  �  �  =     �  l   >  �  �  =     �  \  >  �  �  =     �  q   >  �  �  =     �  A  >  �  �  =        m   O                       >  �    =       p   >      =       o   >      9       h   �  �  �  �  �  �      �         �  =       J  �     	      >  J  	  �  �  �  �  =  �   
  �  �  �     
  �   >  �    �  �  �  �  A      �   �   =         O                     =       m   O                     �           >      =         =       J  �           =       n   O                     �           >      =         A       m   �   =         Q             Q            Q             P     !           �  !  8                main                     FRAGMENT                     PerViewData                                   FRAGMENT0  0           shadow_map_images          0                        FRAGMENT              smp                                   FRAGMENT                                               0@                 @�@     	       smp_depth                                   FRAGMENT                                            0@                @�@                   MaterialDataUbo                                  FRAGMENTP   P           per_material_data       normal_texture                                 FRAGMENT             normal_texture       base_color_texture                                 FRAGMENT             base_color_texture       emissive_texture                                 FRAGMENT             emissive_texture       metallic_roughness_texture                                 FRAGMENT             metallic_roughness_texture       occlusion_texture                                 FRAGMENT             occlusion_texture              PerObjectData                                  FRAGMENT�   �                     
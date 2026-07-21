CREATE TABLE IF NOT EXISTS theme_profiles (
 id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL UNIQUE,
 base_mode TEXT NOT NULL CHECK(base_mode IN('light','dark')), accent TEXT NOT NULL,
 surface_main TEXT NOT NULL, surface_sidebar TEXT NOT NULL, surface_raised TEXT NOT NULL,
 text_primary TEXT NOT NULL, text_secondary TEXT NOT NULL, border_color TEXT NOT NULL,
 radius_px INTEGER NOT NULL DEFAULT 12, font_scale REAL NOT NULL DEFAULT 1.0,
 density TEXT NOT NULL DEFAULT 'comfortable' CHECK(density IN('compact','comfortable','spacious')),
 motion TEXT NOT NULL DEFAULT 'normal' CHECK(motion IN('off','reduced','normal')),
 is_builtin INTEGER NOT NULL DEFAULT 0, is_active INTEGER NOT NULL DEFAULT 0,
 version INTEGER NOT NULL DEFAULT 1, created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
 updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_theme_active ON theme_profiles(is_active) WHERE is_active=1;
INSERT OR IGNORE INTO theme_profiles(name,base_mode,accent,surface_main,surface_sidebar,surface_raised,text_primary,text_secondary,border_color,radius_px,font_scale,density,motion,is_builtin,is_active) VALUES
('Midnight','dark','#8bd8a5','#11130f','#0c0e0b','#20241d','#f4f1e8','#d3cfc3','#31372c',12,1.0,'comfortable','normal',1,1),
('Pure Black OLED','dark','#71e5a4','#000000','#000000','#101210','#ffffff','#c7cbc8','#292d2a',8,1.0,'compact','reduced',1,0),
('Snow','light','#2f8c62','#f7f8fa','#18201c','#ffffff','#171916','#4e5651','#d9dfdc',10,1.0,'comfortable','normal',1,0),
('Warm Paper','light','#a66a35','#f4eee2','#302820','#fffaf0','#2a211a','#66594d','#d8c8b6',14,1.03,'spacious','reduced',1,0),
('Cyber','dark','#00e5ff','#071015','#061c24','#102731','#e9fdff','#94cbd1','#1d5360',4,0.98,'compact','normal',1,0),
('Forest','dark','#8ddf78','#101a13','#0b130d','#1a2a1e','#eff8ee','#bad0bc','#314c36',14,1.0,'comfortable','normal',1,0),
('Ocean','dark','#62b8ff','#0b1724','#08121d','#152b3e','#eff8ff','#b5cce0','#29475e',12,1.0,'comfortable','normal',1,0),
('Nord','dark','#88c0d0','#2e3440','#242933','#3b4252','#eceff4','#d8dee9','#4c566a',8,1.0,'comfortable','reduced',1,0),
('Glass','dark','#a6e3c8','#151a1c','#101416','#263035','#f2f7f6','#c5d0ce','#445055',18,1.0,'spacious','normal',1,0),
('Minimal Gray','light','#555f66','#f1f2f3','#202326','#ffffff','#1f2224','#626a6f','#d4d7d9',6,0.98,'compact','off',1,0),
('High Contrast','dark','#ffdf00','#000000','#000000','#111111','#ffffff','#ffffff','#ffffff',2,1.12,'spacious','off',1,0);

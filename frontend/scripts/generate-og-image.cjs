// Simple script to generate OGP image
// This creates a basic HTML file that can be converted to PNG
// For production, you might want to use a service like Cloudinary or generate it server-side

const fs = require('fs');
const path = require('path');

// Create a simple HTML file that can be used as OGP image
// In production, this would be converted to PNG
const ogImageHTML = `<!DOCTYPE html>
<html>
<head>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body {
      width: 1200px;
      height: 630px;
      background: #000000;
      display: flex;
      flex-direction: column;
      justify-content: center;
      align-items: flex-start;
      padding: 100px;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Arial, sans-serif;
      position: relative;
      overflow: hidden;
    }
    .grid {
      position: absolute;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      background-image: 
        linear-gradient(rgba(26, 26, 26, 0.5) 1px, transparent 1px),
        linear-gradient(90deg, rgba(26, 26, 26, 0.5) 1px, transparent 1px);
      background-size: 40px 40px;
    }
    .radial-lines {
      position: absolute;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
      width: 100%;
      height: 100%;
      opacity: 0.3;
    }
    .radial-lines::before,
    .radial-lines::after {
      content: '';
      position: absolute;
      top: 50%;
      left: 50%;
      width: 200%;
      height: 2px;
      background: #333;
      transform-origin: center;
    }
    .radial-lines::before {
      transform: translate(-50%, -50%) rotate(45deg);
    }
    .radial-lines::after {
      transform: translate(-50%, -50%) rotate(-45deg);
    }
    .content {
      position: relative;
      z-index: 1;
    }
    h1 {
      font-size: 72px;
      font-weight: bold;
      color: #ffffff;
      margin-bottom: 20px;
    }
    .subtitle {
      font-size: 36px;
      color: #cccccc;
      margin-bottom: 20px;
    }
    .description {
      font-size: 28px;
      color: #999999;
    }
  </style>
</head>
<body>
  <div class="grid"></div>
  <div class="radial-lines"></div>
  <div class="content">
    <h1>SubsidyPayment</h1>
    <div class="subtitle">Sponsor the daily-use services your target users rely on.</div>
    <div class="description">Track performance. Pay only for results.</div>
  </div>
</body>
</html>`;

const publicDir = path.join(__dirname, '..', 'public');
if (!fs.existsSync(publicDir)) {
  fs.mkdirSync(publicDir, { recursive: true });
}

// For now, we'll create a simple SVG that works as OGP image
// Most platforms support SVG for OGP images
const ogImageSVG = `<svg width="1200" height="630" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <style>
      .title { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Arial, sans-serif; font-size: 72px; font-weight: bold; fill: #ffffff; }
      .subtitle { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Arial, sans-serif; font-size: 36px; fill: #cccccc; }
      .description { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Arial, sans-serif; font-size: 28px; fill: #999999; }
    </style>
    <pattern id="grid" width="40" height="40" patternUnits="userSpaceOnUse">
      <path d="M 40 0 L 0 0 0 40" fill="none" stroke="#1a1a1a" stroke-width="1"/>
    </pattern>
  </defs>
  <rect width="1200" height="630" fill="#000000"/>
  <rect width="1200" height="630" fill="url(#grid)"/>
  <g opacity="0.3">
    <line x1="600" y1="315" x2="1200" y2="0" stroke="#333333" stroke-width="2"/>
    <line x1="600" y1="315" x2="1200" y2="630" stroke="#333333" stroke-width="2"/>
    <line x1="600" y1="315" x2="0" y2="0" stroke="#333333" stroke-width="2"/>
    <line x1="600" y1="315" x2="0" y2="630" stroke="#333333" stroke-width="2"/>
  </g>
  <g transform="translate(100, 200)">
    <text x="0" y="0" class="title">SubsidyPayment</text>
    <text x="0" y="100" class="subtitle">Sponsor the daily-use services your target users rely on.</text>
    <text x="0" y="180" class="description">Track performance. Pay only for results.</text>
  </g>
</svg>`;

fs.writeFileSync(path.join(publicDir, 'og-image.svg'), ogImageSVG);
console.log('OGP image (SVG) generated successfully!');


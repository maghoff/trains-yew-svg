import init, { run_app } from './pkg/trains_yew_svg.js';
async function main() {
   await init('/pkg/trains_yew_svg_bg.wasm');
   run_app();
}
main()

import init, { run_app } from './pkg/tody_chat_ui.js';
async function main() {
   await init('/pkg/tody_chat_ui_bg.wasm');
   run_app();
}
main()

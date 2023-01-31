export function drag(dom, mousedownCallback, mousemoveCallback, mouseupCallback){
    dom.onmousedown = function(event) {
        mousedownCallback();
        
        let shiftX = event.clientX - dom.getBoundingClientRect().left;
        let shiftY = event.clientY - dom.getBoundingClientRect().top;
        
        let prev_x = undefined;
        let prev_y = undefined;
        function movePosition(pageX, pageY) {
            let x = pageX - shiftX;
            let y = pageY - shiftY;
            prev_x |= x;
            prev_y |= y;
            let dx = x - prev_x;
            let dy = y - prev_y;
            mousemoveCallback(x, y, dx, dy)
        }
        // マウスを動かした時の処理
        function onMouseMove(event) {
            let parent = dom.parentNode;
            movePosition(event.pageX - parent.offsetLeft, event.pageY - parent.offsetTop);
        }
        
        document.addEventListener('mousemove', onMouseMove);

        // マウスを離した時にmousemoveイベントを解除する
        let onMouseUp =() => {
            mouseupCallback();
            document.removeEventListener('mousemove', onMouseMove);
            document.removeEventListener('mouseup', onMouseUp);
        };
        document.addEventListener('mouseup', onMouseUp)
    }
}

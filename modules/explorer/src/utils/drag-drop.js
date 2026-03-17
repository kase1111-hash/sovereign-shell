// Drag and drop handler for the Explorer.

const DragDrop = (() => {
    let draggedPaths = [];

    function init() {
        // Prevent default browser drag behavior on the window
        document.addEventListener('dragover', (e) => {
            e.preventDefault();
            e.dataTransfer.dropEffect = 'copy';
        });

        document.addEventListener('drop', (e) => {
            e.preventDefault();
        });
    }

    function setDragData(paths) {
        draggedPaths = [...paths];
    }

    function getDragData() {
        return draggedPaths;
    }

    function clearDragData() {
        draggedPaths = [];
    }

    function makeDropTarget(element, onDrop) {
        element.addEventListener('dragover', (e) => {
            e.preventDefault();
            e.stopPropagation();
            element.classList.add('drop-target');
            e.dataTransfer.dropEffect = 'move';
        });

        element.addEventListener('dragleave', (e) => {
            element.classList.remove('drop-target');
        });

        element.addEventListener('drop', (e) => {
            e.preventDefault();
            e.stopPropagation();
            element.classList.remove('drop-target');
            onDrop(getDragData());
            clearDragData();
        });
    }

    return { init, setDragData, getDragData, clearDragData, makeDropTarget };
})();

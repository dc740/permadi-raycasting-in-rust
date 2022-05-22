onmessage = function(e) {
  console.log('Worker: Message received from main script. downloading file...');
  var file_path = e.data;
  fetch(file_path)
  .then(resp => resp.blob())
  .then(workerResult => {
    filename_ascii = new Blob([file_path + '|'], {type: 'text/plain'});
    new_blob = new Blob([filename_ascii, workerResult], {type: 'application/octet-stream'});
    return new_blob;    
  })
  .then(complete_blob => complete_blob.arrayBuffer())
  .then(array_buffer => {
    postMessage(array_buffer);
  })
  .catch((error) => console.error(error));
}
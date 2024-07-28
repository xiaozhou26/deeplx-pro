const { exec } = require('child_process');

// Function to start the server
function startServer() {
  const serverProcess = exec('node server.js', (error, stdout, stderr) => {
    if (error) {
      console.error(`Error starting server: ${error.message}`);
      return;
    }
    if (stderr) {
      console.error(`Stderr: ${stderr}`);
      return;
    }
    console.log(`Stdout: ${stdout}`);
  });

  // Set a timeout to restart the server after 30 minutes
  setTimeout(() => {
    console.log('Restarting server...');
    serverProcess.kill('SIGTERM'); // Gracefully kill the server
    startServer(); // Restart the server
  }, 450000); // 30 minutes in milliseconds
}

// Start the server for the first time
startServer();

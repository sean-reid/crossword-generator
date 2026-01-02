// Progress worker - runs independently to animate progress bar
let progressInterval: number | null = null;
let startTime = 0;
let estimatedTime = 1000;

self.onmessage = (event: MessageEvent) => {
  const { type, payload } = event.data;

  switch (type) {
    case 'START':
      startTime = Date.now();
      estimatedTime = payload.estimatedTime || 1000;
      
      if (progressInterval) {
        clearInterval(progressInterval);
      }
      
      progressInterval = setInterval(() => {
        const elapsed = Date.now() - startTime;
        const percent = Math.min(95, (elapsed / estimatedTime) * 100);
        
        const stage = elapsed < estimatedTime * 0.3 ? 'Encoding constraints...' :
                     elapsed < estimatedTime * 0.7 ? 'Solving SAT problem...' :
                     'Extracting solution...';
        
        self.postMessage({ 
          type: 'UPDATE',
          percent: Math.floor(percent),
          stage,
          elapsed,
          estimated: estimatedTime
        });
      }, 100) as unknown as number;
      break;

    case 'UPDATE_ESTIMATE':
      // Update estimate mid-flight
      estimatedTime = payload.estimatedTime;
      break;

    case 'STOP':
      if (progressInterval) {
        clearInterval(progressInterval);
        progressInterval = null;
      }
      break;
  }
};

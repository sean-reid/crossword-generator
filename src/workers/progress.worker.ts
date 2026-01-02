// Progress worker - runs independently to animate progress bar
let progressInterval: number | null = null;
let startTime = 0;
let estimatedTime = 1000;
let encodingCompleteTime = 0;
let actualEncodingTime = 0;

self.onmessage = (event: MessageEvent) => {
  const { type, payload } = event.data;

  switch (type) {
    case 'START':
      startTime = Date.now();
      estimatedTime = payload.estimatedTime || 1000;
      encodingCompleteTime = 0;
      actualEncodingTime = 0;
      
      if (progressInterval) {
        clearInterval(progressInterval);
      }
      
      progressInterval = setInterval(() => {
        const now = Date.now();
        const elapsed = now - startTime;
        
        let percent: number;
        let stage: string;
        
        if (encodingCompleteTime === 0) {
          // Still encoding or waiting for encoding to complete
          percent = Math.min(30, (elapsed / estimatedTime) * 100);
          stage = elapsed < 1000 ? 'Starting...' : 'Encoding constraints...';
        } else {
          // Encoding complete, now solving
          const timeSinceSolveStart = now - encodingCompleteTime;
          const totalElapsed = actualEncodingTime + timeSinceSolveStart;
          percent = Math.min(95, (totalElapsed / estimatedTime) * 100);
          
          stage = timeSinceSolveStart < (estimatedTime - actualEncodingTime) * 0.8 
            ? 'Solving SAT problem...' 
            : 'Extracting solution...';
        }
        
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
      // Update estimate with actual encoding time
      if (payload.actualEncodingTime !== undefined) {
        actualEncodingTime = payload.actualEncodingTime;
        encodingCompleteTime = Date.now();
      }
      estimatedTime = payload.estimatedTime;
      break;

    case 'STOP':
      if (progressInterval) {
        clearInterval(progressInterval);
        progressInterval = null;
      }
      encodingCompleteTime = 0;
      actualEncodingTime = 0;
      break;
  }
};

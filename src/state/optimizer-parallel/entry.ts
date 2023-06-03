import { setupCombinations } from '../optimizer/optimizerSetup';
import { getAffixCombinations, getLayerNumber } from './affixTree';
import { FINISHED, SETUP, START } from './workerMessageTypes';

// should make this a settings variable or something later on
const NUM_THREADS = 1;

function calculate(reduxState: any, isWasm: boolean) {
  console.log('Parallel Optimizer');
  console.log('State', reduxState);

  // get the extra combinations from the redux state
  const combinations = setupCombinations(reduxState);

  if (combinations.length === 0) {
    console.error('No combinations found');
    return;
  }

  const affixArray = combinations[0].settings?.affixesArray;

  if (!affixArray) {
    console.error('No affixes found');
    return;
  }

  const layer = getLayerNumber(affixArray, NUM_THREADS);

  console.log(`Creating ${NUM_THREADS} threads to calculate ${layer} layers`);
  const workers = [...Array(NUM_THREADS)].map((_, index) => {
    return {
      status: 'created',
      workerId: index,
      worker: new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' }),
    };
  });

  const affixcombinations = getAffixCombinations(affixArray, layer);

  // split work into NUM_THREADS chunks, each chunk getting a number of subtrees to calculate
  const chunks = splitCombinations(affixcombinations);

  // send chunks to workers
  workers.forEach(({ worker }, index) => {
    worker.postMessage({
      type: SETUP,
      data: {
        chunks: chunks[index],
        combinations,
      },
    });
  });

  // attach listener
  const results: any[][] = [];
  workers.forEach(({ worker }, index) => {
    worker.onmessage = (e) => {
      console.log('Worker message', e.data);

      if (e.data.type === 'FINISHED') {
        results.push(e.data.data);
        workers[index].status = FINISHED;

        // check if all workers finished
        if (workers.every(({ status }) => status === FINISHED)) {
          console.log('All workers finished');
          console.log('Results', results.flat(1));
          const endTime = performance.now();
          console.log('Time', endTime - startTime, 'ms');
        }
      }
    };
  });

  const startTime = performance.now();

  // start workers
  workers.forEach(({ worker }) => {
    worker.postMessage({
      type: START,
      data: {
        isWasm,
      },
    });
  });
}

/**
 * Splits the work into NUM_THREADS chunks, each chunk getting an array of subtrees to calculate
 * combinations are split in a round-robin fashion
 *
 * @param combinations all possible affix combinations
 */
function splitCombinations<T>(combinations: T[][]) {
  const chunks = [...Array(NUM_THREADS)].map(() => [] as T[][]);

  let chunkIndex = 0;
  combinations.forEach((combination) => {
    chunks[chunkIndex].push(combination);
    chunkIndex = (chunkIndex + 1) % NUM_THREADS;
  });

  return chunks;
}

export default calculate;

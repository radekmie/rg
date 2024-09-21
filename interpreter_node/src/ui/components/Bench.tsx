import {
  Button,
  Card,
  Elevation,
  FormGroup,
  Intent,
  Label,
  NumericInput,
} from '@blueprintjs/core';
import { useCallback, useMemo, useState } from 'react';

import * as rg from '../../rg';
import { useNumericState } from '../hooks/useNumericState';
import * as styles from '../index.module.css';

type BenchBlockProps = {
  action: (
    gameDeclaration: rg.ast.GameDeclaration,
    value: number,
    logger: rg.ast.Logger,
  ) => Promise<void>;
  actionText: string;
  gameDeclaration: rg.ast.GameDeclaration;
  id?: string;
  initialValue: number;
  label: string;
  labelInfo?: string;
  logsLimit: number;
  max?: number;
  min?: number;
};

function BenchBlock({
  action,
  actionText,
  gameDeclaration,
  id,
  initialValue,
  label,
  labelInfo,
  logsLimit,
  max,
  min,
}: BenchBlockProps) {
  const inputState = useNumericState(initialValue);

  const [logs, setLogs] = useState<string[]>([]);
  const logger = useMemo(
    () => ({
      log(message: string) {
        message = `[${new Date().toJSON()}] ${message}`;
        setLogs(logs => logs.concat(message).slice(-logsLimit));
      },
    }),
    [logsLimit],
  );

  const [result, setResult] = useState<Intent>(Intent.NONE);
  const onAction = useCallback(() => {
    async function run() {
      setLogs([]);
      setResult(Intent.PRIMARY);
      try {
        await action(gameDeclaration, inputState.valueAsNumber, logger);
        setResult(Intent.SUCCESS);
      } catch (error) {
        setResult(Intent.DANGER);
        console.error(error);
      }
    }

    // eslint-disable-next-line @typescript-eslint/no-floating-promises -- Run it in background.
    run();
  }, [action, gameDeclaration, inputState.valueAsNumber, logger]);

  return (
    <Card className={styles.block} elevation={Elevation.TWO}>
      <FormGroup label={label} labelFor={id} labelInfo={labelInfo}>
        <NumericInput
          disabled={result === Intent.PRIMARY}
          id={id}
          max={max}
          min={min}
          onValueChange={inputState.onValueChange}
          value={inputState.valueAsString}
        />
      </FormGroup>

      <Button
        disabled={result === Intent.PRIMARY}
        intent={result}
        onClick={onAction}
      >
        {actionText}
      </Button>

      <pre>{logs.join('\n')}</pre>
    </Card>
  );
}

export type BenchProps = {
  gameDeclaration: rg.ast.GameDeclaration;
  stats: string;
};

export function Bench({ gameDeclaration, stats }: BenchProps) {
  return (
    <section className={styles.wrapScroll}>
      <Card className={styles.block} elevation={Elevation.TWO}>
        <Label>Statistics</Label>
        <pre>{stats}</pre>
      </Card>
      <BenchBlock
        action={rg.ast.run}
        actionText="Run"
        gameDeclaration={gameDeclaration}
        id="iterations"
        initialValue={100}
        label="Iterations"
        labelInfo="(number of random games to play)"
        logsLimit={18}
        max={1000}
        min={1}
      />
      <BenchBlock
        action={rg.ast.perf}
        actionText="Bench"
        gameDeclaration={gameDeclaration}
        id="depth"
        initialValue={3}
        label="Maximum depth"
        labelInfo="(game tree depth to calculate)"
        logsLimit={100}
        max={10}
        min={0}
      />
    </section>
  );
}

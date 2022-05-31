import {
  Button,
  Card,
  Elevation,
  FormGroup,
  Intent,
  NumericInput,
} from '@blueprintjs/core';
import { useCallback, useMemo, useState } from 'react';

import * as rg from '../../rg';
import { useNumericState } from '../hooks/useNumericState';
import * as styles from '../index.module.css';

type BenchBlockProps = {
  action: (
    game: rg.ist.Game,
    value: number,
    logger: rg.ist.Logger,
  ) => Promise<void>;
  actionText: string;
  game: rg.ist.Game;
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
  game,
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
    setLogs([]);
    setResult(Intent.PRIMARY);
    action(game, inputState.valueAsNumber, logger).then(
      () => {
        setResult(Intent.SUCCESS);
      },
      () => {
        setResult(Intent.DANGER);
      },
    );
  }, [action, game, inputState.valueAsNumber, logger]);

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

export type BenchProps = { game: rg.ist.Game };

export function Bench({ game }: BenchProps) {
  return (
    <section className={styles.wrapScroll}>
      <BenchBlock
        action={rg.ist.run}
        actionText="Run"
        game={game}
        id="iterations"
        initialValue={100}
        label="Iterations"
        labelInfo="(number of random games to play)"
        logsLimit={12}
        max={1000}
        min={1}
      />
      <BenchBlock
        action={rg.ist.perf}
        actionText="Bench"
        game={game}
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

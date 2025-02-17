import {
  Button,
  Card,
  ControlGroup,
  Elevation,
  FormGroup,
  HTMLSelect,
  InputGroup,
  Intent,
  Label,
  NumericInput,
} from '@blueprintjs/core';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { usePromise } from 'ui/hooks/usePromise';

import { RgGameDeclaration } from '../../parse';
import { localeCompare, prettyError, random } from '../../utils';
import * as wasm from '../../wasm';
import { useNumericState } from '../hooks/useNumericState';
import * as styles from '../index.module.css';

type BenchBlockProps = {
  action: (
    gameDeclaration: RgGameDeclaration,
    value: number,
    logger: wasm.Logger,
  ) => Promise<void>;
  actionText: string;
  gameDeclaration: RgGameDeclaration;
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
        logger.log(prettyError(error));
      }
    }

    // eslint-disable-next-line @typescript-eslint/no-floating-promises -- Run it in background.
    run();
  }, [action, gameDeclaration, inputState.valueAsNumber, logger]);

  return (
    <Card className={styles.block} compact elevation={Elevation.TWO}>
      <FormGroup label={label} labelFor={id} labelInfo={labelInfo}>
        <ControlGroup>
          <Button
            disabled={result === Intent.PRIMARY}
            intent={result}
            onClick={onAction}
          >
            {actionText}
          </Button>
          <NumericInput
            disabled={result === Intent.PRIMARY}
            fill
            id={id}
            max={max}
            min={min}
            onValueChange={inputState.onValueChange}
            value={inputState.valueAsString}
          />
        </ControlGroup>
      </FormGroup>

      <pre>{logs.join('\n')}</pre>
    </Card>
  );
}

export type PlayBlockProps = {
  gameDeclaration: RgGameDeclaration;
};

function PlayBlock({ gameDeclaration }: PlayBlockProps) {
  const [path, setPath] = useState('/');
  useEffect(() => setPath('/'), [gameDeclaration]);

  const result = usePromise(
    () => wasm.apply(gameDeclaration, path),
    [gameDeclaration, path],
  );

  return (
    <Card className={styles.block} compact elevation={Elevation.TWO}>
      <FormGroup
        label="Path"
        labelFor="path"
        labelInfo="(slash separates moves; space separates tags)"
      >
        <ControlGroup>
          <Button
            icon="double-chevron-left"
            disabled={path === '/'}
            onClick={() => setPath('/')}
          />
          <Button
            icon="chevron-left"
            disabled={path === '/'}
            onClick={() =>
              setPath(path => `${path.split('/').slice(0, -2).join('/')}/`)
            }
          />
          <InputGroup fill id="path" onValueChange={setPath} value={path} />
          <HTMLSelect
            disabled={!result.value?.moves.length}
            id="move"
            onChange={({ currentTarget: { value } }) =>
              setPath(path => `${path}${value}/`)
            }
            options={[
              { disabled: true, label: 'Move', value: '(default)' },
              ...(result.value?.moves.sort(localeCompare) ?? []),
            ]}
            value="(default)"
          />
          <Button
            icon="random"
            disabled={!result.value?.moves.length}
            onClick={() =>
              result.value &&
              setPath(path => `${path}${random(result.value.moves)}/`)
            }
          />
        </ControlGroup>
      </FormGroup>
      <pre>
        {result.error ? prettyError(result.error) : result.value?.state}
      </pre>
    </Card>
  );
}

export type BenchProps = {
  gameDeclaration: RgGameDeclaration;
  stats: string;
};

export function Bench({ gameDeclaration, stats }: BenchProps) {
  return (
    <section className={styles.wrapScroll}>
      <Card className={styles.block} compact elevation={Elevation.TWO}>
        <Label>Statistics</Label>
        <pre>{stats}</pre>
      </Card>
      <BenchBlock
        action={wasm.run}
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
        action={wasm.perf}
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
      <PlayBlock gameDeclaration={gameDeclaration} />
    </section>
  );
}

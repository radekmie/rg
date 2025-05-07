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
import {
  Dispatch,
  SetStateAction,
  useCallback,
  useEffect,
  useMemo,
  useState,
} from 'react';
import { AsyncState, usePromise } from 'ui/hooks/usePromise';

import { RgGameDeclaration } from '../../parse';
import { localeCompare, prettyError, random } from '../../utils';
import * as wasm from '../../wasm';
import { useNumericState } from '../hooks/useNumericState';
import * as styles from '../index.module.css';

type InitialState = AsyncState<Awaited<ReturnType<typeof wasm.apply>>, unknown>;

type BenchBlockProps = {
  action: (
    gameDeclaration: RgGameDeclaration,
    initialStatePath: string,
    value: number,
    logger: wasm.Logger,
  ) => Promise<void>;
  actionText: string;
  gameDeclaration: RgGameDeclaration;
  id?: string;
  initialState: InitialState;
  initialStatePath: string;
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
  initialState,
  initialStatePath,
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
  const disabled =
    result === Intent.PRIMARY ||
    initialState.loading ||
    !initialState.value?.moves.length;

  const onAction = useCallback(() => {
    if (disabled) {
      return;
    }

    async function run() {
      setLogs([]);
      setResult(Intent.PRIMARY);
      try {
        await action(
          gameDeclaration,
          initialStatePath,
          inputState.valueAsNumber,
          logger,
        );
        setResult(Intent.SUCCESS);
      } catch (error) {
        setResult(Intent.DANGER);
        logger.log(prettyError(error));
      }
    }

    // eslint-disable-next-line @typescript-eslint/no-floating-promises -- Run it in background.
    run();
  }, [
    action,
    disabled,
    gameDeclaration,
    initialStatePath,
    inputState.valueAsNumber,
    logger,
  ]);

  return (
    <Card className={styles.block} compact elevation={Elevation.TWO}>
      <FormGroup label={label} labelFor={id} labelInfo={labelInfo}>
        <ControlGroup>
          <Button disabled={disabled} intent={result} onClick={onAction}>
            {actionText}
          </Button>
          <NumericInput
            disabled={disabled}
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
  initialState: InitialState;
  initialStatePath: string;
  setInitialStatePath: Dispatch<SetStateAction<string>>;
};

function PlayBlock({
  initialState,
  initialStatePath,
  setInitialStatePath,
}: PlayBlockProps) {
  const [isAuto, setIsAuto] = useState(false);
  useEffect(() => {
    const timeout = setTimeout(() => {
      if (isAuto && initialState.value?.moves.length) {
        setInitialStatePath(
          path => `${path}${random(initialState.value.moves)}/`,
        );
      } else {
        setIsAuto(false);
      }
    });

    return () => clearTimeout(timeout);
  }, [isAuto, initialState.value, setInitialStatePath]);

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
            disabled={isAuto || initialStatePath === '/'}
            onClick={() => setInitialStatePath('/')}
          />
          <Button
            icon="chevron-left"
            disabled={isAuto || initialStatePath === '/'}
            onClick={() =>
              setInitialStatePath(
                path => `${path.split('/').slice(0, -2).join('/')}/`,
              )
            }
          />
          <InputGroup
            disabled={isAuto}
            fill
            id="path"
            onValueChange={setInitialStatePath}
            value={initialStatePath}
          />
          <HTMLSelect
            disabled={
              isAuto ||
              initialState.loading ||
              !initialState.value?.moves.length
            }
            iconName="caret-down"
            id="move"
            onChange={({ currentTarget: { value } }) =>
              setInitialStatePath(path => `${path}${value}/`)
            }
            options={[
              { disabled: true, label: 'Move', value: '(default)' },
              ...(initialState.value?.moves.sort(localeCompare) ?? []),
            ]}
            value="(default)"
          />
          <Button
            icon="random"
            intent={
              isAuto
                ? Intent.PRIMARY
                : initialState.error
                  ? Intent.DANGER
                  : initialState.value?.isFinal
                    ? Intent.SUCCESS
                    : undefined
            }
            disabled={
              isAuto ||
              initialState.loading ||
              !initialState.value?.moves.length
            }
            onClick={() =>
              initialState.value?.moves.length &&
              setInitialStatePath(
                path => `${path}${random(initialState.value.moves)}/`,
              )
            }
          />
          <Button
            icon="double-chevron-right"
            intent={
              isAuto
                ? Intent.PRIMARY
                : initialState.error
                  ? Intent.DANGER
                  : initialState.value?.isFinal
                    ? Intent.SUCCESS
                    : undefined
            }
            disabled={
              isAuto ||
              initialState.loading ||
              !initialState.value?.moves.length
            }
            onClick={() => setIsAuto(true)}
          />
        </ControlGroup>
      </FormGroup>
      <pre>
        {initialState.error
          ? prettyError(initialState.error)
          : initialState.value?.state}
      </pre>
    </Card>
  );
}

export type BenchProps = { gameDeclaration: RgGameDeclaration; stats: string };
export function Bench({ gameDeclaration, stats }: BenchProps) {
  const [path, setPath] = useState('/');
  const initialState = usePromise(
    () => wasm.apply(gameDeclaration, path),
    [gameDeclaration, path],
  );

  useEffect(() => setPath('/'), [gameDeclaration]);

  return (
    <section className={styles.wrapScroll}>
      <Card className={styles.block} compact elevation={Elevation.TWO}>
        <Label>Statistics</Label>
        <pre>{stats}</pre>
      </Card>
      <PlayBlock
        initialState={initialState}
        initialStatePath={path}
        setInitialStatePath={setPath}
      />
      <BenchBlock
        action={wasm.run}
        actionText="Run"
        gameDeclaration={gameDeclaration}
        id="iterations"
        initialState={initialState}
        initialStatePath={path}
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
        initialState={initialState}
        initialStatePath={path}
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

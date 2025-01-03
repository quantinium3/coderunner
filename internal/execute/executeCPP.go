package execute

import (
	"context"
	"fmt"
	"io"
	"os/exec"
	"time"

	comphubtypes "github.com/quantinium03/comphub/internal/comphubTypes"
)

func ExecuteCPP(filename string, stdinputs []string) (comphubtypes.ExecutionRes, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 10 * time.Second)
	defer cancel()

	executable := "execution_zone/exec_binary"
	compCmd := exec.CommandContext(ctx, "g++", "-o", executable, filename)

	compOut, err := compCmd.CombinedOutput()
	if err != nil {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error: fmt.Sprintf("Compilation Failed :%v\n%s", err, string(compOut)),
		}, err
	}

	cmd := exec.CommandContext(ctx, "./" + executable)

	input, err := cmd.StdinPipe()
	if err != nil {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error: fmt.Sprintf("Failed to create the stdin pipe %v", err),
		}, err
	}
	output, err := cmd.StdoutPipe()
	if err != nil {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error: fmt.Sprintf("Failed to create the stdout pipe %v", err),
		}, err
	}
	stderr, err := cmd.StderrPipe()
	if err != nil {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error: fmt.Sprintf("Failed to create the stderr pipe %v", err),
		}, err
	}

	if err := cmd.Start(); err != nil {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error: fmt.Sprintf("Failed to start command: %v", err),
		},err
	}

	go func() {
		defer input.Close()
		for _, stdinput := range stdinputs {
			if _, err := input.Write([]byte(stdinput + "\n")); err != nil {
				cancel()
				return
			}
		}
	}()

	stdoutBytes, err := io.ReadAll(output)
	if err != nil {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error: fmt.Sprintf("Failed to read stdout: %v", err),
		}, err
	}

	stderrBytes, err := io.ReadAll(stderr)
	if err != nil {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error: fmt.Sprintf("Failed to read the stderr : %v", err),
		}, err
	}

	if err := cmd.Wait(); err != nil {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error: fmt.Sprintf("Command execution failed: %v", err),
		}, err
	}

	if len(stderrBytes) > 0  {
		return comphubtypes.ExecutionRes{
			Success: false,
			Error: string(stderrBytes),
		}, fmt.Errorf("stderr: %v", string(stderrBytes))
	}

	return comphubtypes.ExecutionRes{
		Success: true,
		Output: string(stdoutBytes),
	}, nil
}

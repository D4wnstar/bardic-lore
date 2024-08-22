import type { CustomThemeConfig } from '@skeletonlabs/tw-plugin'

export const cosmos: CustomThemeConfig = {
	name: 'cosmos',
	properties: {
		// =~= Theme Properties =~=
		'--theme-font-family-base': `system-ui`,
		'--theme-font-family-heading': `Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, 'Noto Sans', sans-serif, 'Apple Color Emoji', 'Segoe UI Emoji', 'Segoe UI Symbol', 'Noto Color Emoji'`,
		'--theme-font-color-base': '16 48 90',
		'--theme-font-color-dark': '211 223 241',
		'--theme-rounded-base': '8px',
		'--theme-rounded-container': '8px',
		'--theme-border-base': '1px',
		// =~= Theme On-X Colors =~=
		'--on-primary': '255 255 255',
		'--on-secondary': '255 255 255',
		'--on-tertiary': '0 0 0',
		'--on-success': '0 0 0',
		'--on-warning': '0 0 0',
		'--on-error': '255 255 255',
		'--on-surface': '211 223 241',
		// =~= Theme Colors  =~=
		// primary | #2161b8
		'--color-primary-50': '222 231 244', // #dee7f4
		'--color-primary-100': '211 223 241', // #d3dff1
		'--color-primary-200': '200 216 237', // #c8d8ed
		'--color-primary-300': '166 192 227', // #a6c0e3
		'--color-primary-400': '100 144 205', // #6490cd
		'--color-primary-500': '33 97 184', // #2161b8
		'--color-primary-600': '30 87 166', // #1e57a6
		'--color-primary-700': '25 73 138', // #19498a
		'--color-primary-800': '20 58 110', // #143a6e
		'--color-primary-900': '16 48 90', // #10305a
		// secondary | #0c96b8
		'--color-secondary-50': '219 239 244', // #dbeff4
		'--color-secondary-100': '206 234 241', // #ceeaf1
		'--color-secondary-200': '194 229 237', // #c2e5ed
		'--color-secondary-300': '158 213 227', // #9ed5e3
		'--color-secondary-400': '85 182 205', // #55b6cd
		'--color-secondary-500': '12 150 184', // #0c96b8
		'--color-secondary-600': '11 135 166', // #0b87a6
		'--color-secondary-700': '9 113 138', // #09718a
		'--color-secondary-800': '7 90 110', // #075a6e
		'--color-secondary-900': '6 74 90', // #064a5a
		// tertiary | #5729c0
		'--color-tertiary-50': '230 223 246', // #e6dff6
		'--color-tertiary-100': '221 212 242', // #ddd4f2
		'--color-tertiary-200': '213 202 239', // #d5caef
		'--color-tertiary-300': '188 169 230', // #bca9e6
		'--color-tertiary-400': '137 105 211', // #8969d3
		'--color-tertiary-500': '87 41 192', // #5729c0
		'--color-tertiary-600': '78 37 173', // #4e25ad
		'--color-tertiary-700': '65 31 144', // #411f90
		'--color-tertiary-800': '52 25 115', // #341973
		'--color-tertiary-900': '43 20 94', // #2b145e
		// success | #1ec45c
		'--color-success-50': '221 246 231', // #ddf6e7
		'--color-success-100': '210 243 222', // #d2f3de
		'--color-success-200': '199 240 214', // #c7f0d6
		'--color-success-300': '165 231 190', // #a5e7be
		'--color-success-400': '98 214 141', // #62d68d
		'--color-success-500': '30 196 92', // #1ec45c
		'--color-success-600': '27 176 83', // #1bb053
		'--color-success-700': '23 147 69', // #179345
		'--color-success-800': '18 118 55', // #127637
		'--color-success-900': '15 96 45', // #0f602d
		// warning | #df7913
		'--color-warning-50': '250 235 220', // #faebdc
		'--color-warning-100': '249 228 208', // #f9e4d0
		'--color-warning-200': '247 222 196', // #f7dec4
		'--color-warning-300': '242 201 161', // #f2c9a1
		'--color-warning-400': '233 161 90', // #e9a15a
		'--color-warning-500': '223 121 19', // #df7913
		'--color-warning-600': '201 109 17', // #c96d11
		'--color-warning-700': '167 91 14', // #a75b0e
		'--color-warning-800': '134 73 11', // #86490b
		'--color-warning-900': '109 59 9', // #6d3b09
		// error | #d81672
		'--color-error-50': '249 220 234', // #f9dcea
		'--color-error-100': '247 208 227', // #f7d0e3
		'--color-error-200': '245 197 220', // #f5c5dc
		'--color-error-300': '239 162 199', // #efa2c7
		'--color-error-400': '228 92 156', // #e45c9c
		'--color-error-500': '216 22 114', // #d81672
		'--color-error-600': '194 20 103', // #c21467
		'--color-error-700': '162 17 86', // #a21156
		'--color-error-800': '130 13 68', // #820d44
		'--color-error-900': '106 11 56', // #6a0b38
		// surface | #232d53
		'--color-surface-50': '222 224 229', // #dee0e5
		'--color-surface-100': '211 213 221', // #d3d5dd
		'--color-surface-200': '200 203 212', // #c8cbd4
		'--color-surface-300': '167 171 186', // #a7abba
		'--color-surface-400': '101 108 135', // #656c87
		'--color-surface-500': '35 45 83', // #232d53
		'--color-surface-600': '32 41 75', // #20294b
		'--color-surface-700': '26 34 62', // #1a223e
		'--color-surface-800': '21 27 50', // #151b32
		'--color-surface-900': '17 22 41' // #111629
	}
}
